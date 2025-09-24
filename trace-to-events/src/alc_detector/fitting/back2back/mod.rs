use crate::{alc_detector::fitting::{Accumulator, Model}, pulse_detection::Real};
use std::f64::consts;

const MAX_EXP: f64 = 30.0; // exp(30) ~ 1e13, safe for double
const MIN_EXP: f64 = -30.0; // exp(-30) ~ 1e-13
const FRAC_SQRT_2_SQRT_PI : f64 = std::f64::consts::FRAC_1_SQRT_2 * std::f64::consts::FRAC_2_SQRT_PI;

/// A back-to-back exponential convolved with a gaussian, and scaled by `i`,
/// that is
/// ```latex
/// f(x) = i\int G(x - x0 - t)E(t)dt,
/// G(t) = \frac{1}{\sqrt{2\pi}s}\exp(-\frac{t^2}{2s^2}),
/// E(t) = 2N\exp(at) if t <= 0 and E(t) = 2N\exp(-bt) if t > 0,
/// N = \frac{ab}{2(a + b)}.
/// ```
/// The result of this convolution is:
/// ```latex
/// f(x) = iN(e^{u(x)} \erfc(y(x)) + e^{v(x)} \erfc(z(x))),
/// u(x) = a(as^2 + 2(x - x0)),
/// v(x) = b(bs^2 - 2(x - x0)),
/// y(x) = \frac{as^2 + (x - x0)}{\sqrt{2}s}
/// z(x) = \frac{bs^2 - (x - x0)}{\sqrt{2}s}
/// ```
pub(crate) struct Back2BackExpParams {
    /// The Integrated Intensity, i.e. the integral of `f(x)` is `i`.
    pub(crate) i: Real,
    /// Exponential rise
    pub(crate) a: Real,
    /// Exponential decay
    pub(crate) b: Real,
    /// Peak location
    pub(crate) x0: Real,
    /// Standard Deviation of the Gaussian
    pub(crate) s: Real,
}

impl Model for Back2BackExpParams {
    type Context = (Real,Real);
    type Accumulator = Back2BackExp;
    
    fn init_parameters(context: &Self::Context) -> Vec<Real> {
        vec![1.0, 1.0, 1.0, (context.0 + context.1)/2.0, 1.0]
    }

    fn accumulator(self) -> Self::Accumulator {
        let s2 = self.s.powi(2);
        let sqrt_2s2 = consts::SQRT_2*self.s;
        let norm_factor = self.a * self.b / (2.0 * (self.a + self.b));
        Self::Accumulator { params: self, s2, sqrt_2s2, norm_factor, }
    }


    fn new(source: &[Real]) -> Self {
        assert_eq!(source.len(), 5);
        Self {
            i: source[0],
            a: source[1],
            b: source[2],
            x0: source[3],
            s: source[4],
        }
    }

    fn lower_bounds(context: &Self::Context) -> Vec<Option<f64>> {
        vec![Some(Real::EPSILON), Some(Real::EPSILON), Some(Real::EPSILON), Some(context.0), Some(Real::EPSILON)]
    }
    
    fn upper_bounds(context: &Self::Context) -> Vec<Option<f64>> {
        vec![None, None, None, Some(context.1), None]
    }
}

pub(crate) struct Back2BackExp {
    params: Back2BackExpParams,
    s2: Real,
    sqrt_2s2: Real,
    norm_factor: Real,
}

impl Back2BackExp {
    fn calc_b2b_arm(&self, arm_coef: f64, x_diff: f64) -> Real {
        let arg_exp = arm_coef / 2.0 * (arm_coef * self.s2 - 2.0 * x_diff);
        let exp = arg_exp.clamp(MIN_EXP, MAX_EXP) // clip to avoid overflow/underflow
            .exp();
        let arg_erfc = (arm_coef * self.s2 - x_diff) / self.sqrt_2s2;
        let erfc = libm::erfc(arg_erfc);
        exp*erfc
    }
    
    fn exp_arm_coef(&self, arm_coef: f64, diff_x: f64) -> Back2BackJacobianArm {
        let exp = (0.5 * arm_coef * (arm_coef * self.s2 + 2.0 * diff_x))
            .clamp(MIN_EXP, MAX_EXP) // clip to avoid overflow/underflow
            .exp();

        let greek_letter = (arm_coef * self.s2 + diff_x) / self.sqrt_2s2;
        let erfc = libm::erfc(greek_letter);
        let g = (-greek_letter*greek_letter)
            .clamp(MIN_EXP, MAX_EXP) // clip to avoid overflow/underflow
            .exp();

        Back2BackJacobianArm { exp, erfc, g }
    }
    
    fn partial_deriv_wrt_arm_coef(&self, arm_coef: f64, arm: &Back2BackJacobianArm, other_arm_coef: f64, diff_x: f64, total_exp_erfc: f64) -> f64 {
        let term_erfc = (arm_coef * self.s2) / 2.0
            + 0.5 * (arm_coef * self.s2 + 2.0 * diff_x);
        let temp = total_exp_erfc / (arm_coef + other_arm_coef);
        self.params.i * (
            other_arm_coef * temp / 2.0
            + self.norm_factor * (
                arm.exp * (
                    term_erfc * arm.erfc
                    - arm.g * FRAC_SQRT_2_SQRT_PI * self.params.s
                )
                - temp
            )
        )
    }
}

struct Back2BackJacobianArm {
    exp: f64,
    erfc: f64,
    g: f64,
}


impl Accumulator for Back2BackExp {
    fn accumulate_value(&self, input: &[Real], output: &mut [Real]) {
        for (i, x) in input.iter().enumerate() {
            let diff = x - self.params.x0;
            let val1 = self.calc_b2b_arm(self.params.a, diff);
            let val2 = self.calc_b2b_arm(self.params.b, diff);
            output[i] = self.params.i*(val1 + val2)*self.norm_factor;
        }
    }

    fn accumulate_jacobian(&self, input: &[Real], output: &mut [&mut [Real]]) {
        for (i, x) in input.iter().enumerate() {
            let diff_x = x - self.params.x0;
            
            // Arm 1 (left)
            let arm_a = self.exp_arm_coef(self.params.a, diff_x);
            let arm_b = self.exp_arm_coef(self.params.b, diff_x);

            // Common factor
            let total_exp_erfc = arm_a.exp * arm_a.erfc + arm_b.exp * arm_b.erfc;

            // df/dI
            let di = total_exp_erfc * self.norm_factor;

            // df/da
            let da = self.partial_deriv_wrt_arm_coef(self.params.a, &arm_a, self.params.b, diff_x, total_exp_erfc);

            // df/db
            let db = self.partial_deriv_wrt_arm_coef(self.params.b, &arm_b, self.params.a, diff_x, total_exp_erfc);

            // df/dx0
            let dx0 = self.params.i * self.norm_factor
                        * (- self.params.a * arm_a.exp * arm_a.erfc + self.params.b * arm_b.exp * arm_b.erfc
                            + FRAC_SQRT_2_SQRT_PI * (arm_a.exp * arm_a.g / self.params.s - arm_b.exp * arm_b.g / self.params.s) );

            // df/ds
            let term1 = -2.0 * arm_a.exp * arm_a.g * (consts::SQRT_2 * self.params.a - (self.params.a * self.s2 + diff_x) / self.sqrt_2s2) * consts::FRAC_2_SQRT_PI / 2.0;
            let term2 = -2.0 * arm_b.exp * arm_b.g * (consts::SQRT_2 * self.params.b - (self.params.b * self.s2 - diff_x) / self.sqrt_2s2) * consts::FRAC_2_SQRT_PI / 2.0;
            let term3 = self.params.a * self.params.a * self.params.s * arm_a.exp * arm_a.erfc;
            let term4 = self.params.b * self.params.b * self.params.s * arm_b.exp * arm_b.erfc;
            let ds = self.params.i * self.norm_factor * (term1 + term2 + term3 + term4);

            // Jacobian
            output[i][0] = di;
            output[i][1] = da;
            output[i][2] = db;
            output[i][3] = dx0;
            output[i][4] = ds;
        }
    }
}