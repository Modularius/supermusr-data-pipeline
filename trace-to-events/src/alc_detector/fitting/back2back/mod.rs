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
pub(crate) struct Back2BackExp {
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

struct Back2BackJacobianArm {
    exp: f64,
    erfc: f64,
    g: f64,
}

struct CalcArmCoef {
    s2: f64,
    sqrt_2s2: f64,
}

impl CalcArmCoef {
    fn calc(&self, arm_coef: f64, diff_x: f64) -> Back2BackJacobianArm {
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
}

fn calc_b2b_arm(arm_coef: f64, x_diff: f64, s2: f64, sqrt_2s2: f64) -> Real {
    let arg_exp = arm_coef / 2.0 * (arm_coef * s2 - 2.0 * x_diff);
    let exp = arg_exp.clamp(MIN_EXP, MAX_EXP) // clip to avoid overflow/underflow
        .exp();
    let arg_erfc = (arm_coef * s2 - x_diff) / sqrt_2s2;
    let erfc = libm::erfc(arg_erfc);
    exp*erfc
}

/// Encapsulates the constants needed to compute `df/da`, where a is an "arm" coefficent,
struct PartialDerivativeWrtArmCoef {
    i: f64,
    s: f64,
    s2: f64,
    norm_factor: f64,
}

impl PartialDerivativeWrtArmCoef {
    fn calc(&self, arm_coef: f64, arm: &Back2BackJacobianArm, other_arm_coef: f64, diff_x: f64, total_exp_erfc: f64) -> f64 {
        let term_erfc = (arm_coef * self.s2) / 2.0
            + 0.5 * (arm_coef * self.s2 + 2.0 * diff_x);
        let temp = total_exp_erfc / (arm_coef + other_arm_coef);
        self.i * (
            other_arm_coef * temp / 2.0
            + self.norm_factor * (
                arm.exp * (
                    term_erfc * arm.erfc
                    - arm.g * FRAC_SQRT_2_SQRT_PI * self.s
                )
                - temp
            )
        )
    }
}

impl Accumulator for Back2BackExp {
    fn accumulate_value(&self, input: &[Real], output: &mut [Real]) {
        let s2 = self.s * self.s;
        let sqrt_2s2 = Real::sqrt(2.0) * self.s;

        let norm_factor = self.a * self.b / (2.0 * (self.a + self.b));

        for (i,x) in input.iter().enumerate() {
            let diff = x - self.x0;
            let val1 = calc_b2b_arm(self.a, diff, s2, sqrt_2s2);
            let val2 = calc_b2b_arm(self.b, diff, s2, sqrt_2s2);
            output[i] = self.i*(val1 + val2)*norm_factor;
        }
    }

    fn accumulate_jacobian(&self, input: &[Real], output: &mut [&mut [Real]]) {
        let s2 = self.s * self.s;
        let sqrt_2s2 = Real::sqrt(2.0) * self.s;
        let norm_factor = self.a * self.b / (2.0 * (self.a + self.b));

        let partial_derivative_wrt_arm_coef = PartialDerivativeWrtArmCoef { i: self.i, s: self.s, s2, norm_factor};
        let calc_arm_coef_value_const = CalcArmCoef { s2, sqrt_2s2 };

        for (i,x) in input.iter().enumerate() {
            let diff_x = x - self.x0;
            
            // Arm 1 (left)
            let arm_a = calc_arm_coef_value_const.calc(self.a, diff_x);
            let arm_b = calc_arm_coef_value_const.calc(self.b, diff_x);

            // Common factor
            let total_exp_erfc = arm_a.exp * arm_a.erfc + arm_b.exp * arm_b.erfc;

            // df/dI
            let di = total_exp_erfc * norm_factor;

            // df/da
            let da = partial_derivative_wrt_arm_coef.calc(self.a, &arm_a, self.b, diff_x, total_exp_erfc);

            // df/db
            let db = partial_derivative_wrt_arm_coef.calc(self.b, &arm_b, self.a, diff_x, total_exp_erfc);

            // df/dx0
            let dx0 = self.i * norm_factor
                        * (- self.a * arm_a.exp * arm_a.erfc + self.b * arm_b.exp * arm_b.erfc
                            + FRAC_SQRT_2_SQRT_PI * (arm_a.exp * arm_a.g / self.s - arm_b.exp * arm_b.g / self.s) );

            // df/ds
            let term1 = -2.0 * arm_a.exp * arm_a.g * (consts::SQRT_2 * self.a - (self.a * s2 + diff_x) / (consts::SQRT_2 * s2)) * consts::FRAC_2_SQRT_PI / 2.0;
            let term2 = -2.0 * arm_b.exp * arm_b.g * (consts::SQRT_2 * self.b - (self.b * s2 - diff_x) / (consts::SQRT_2 * s2)) * consts::FRAC_2_SQRT_PI / 2.0;
            let term3 = self.a * self.a * self.s * arm_a.exp * arm_a.erfc;
            let term4 = self.b * self.b * self.s * arm_b.exp * arm_b.erfc;
            let ds = self.i * norm_factor * (term1 + term2 + term3 + term4);

            // Jacobian
            output[i][0] = di;
            output[i][1] = da;
            output[i][2] = db;
            output[i][3] = dx0;
            output[i][4] = ds;
        }
    }
}

impl Model for Back2BackExp {
    type Context = (Real,Real);
    
    fn init_parameters(context: &Self::Context) -> Vec<Real> {
        vec![1.0, 1.0, 1.0, (context.0 + context.1)/2.0, 1.0]
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
        vec![Some(Real::EPSILON), Some(Real::EPSILON), Some(Real::EPSILON), Some(context.1), Some(Real::EPSILON)]
    }
}