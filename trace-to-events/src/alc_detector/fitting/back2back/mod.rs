use std::marker::PhantomData;

use crate::{alc_detector::fitting::{Accumulator, Model}, pulse_detection::Real};

const MAX_EXP: f64 = 30.0; // exp(30) ~ 1e13, safe for double
const MIN_EXP: f64 = -30.0; // exp(-30) ~ 1e-13
const FRAC_SQRT_2_SQRT_PI : f64 = std::f64::consts::FRAC_1_SQRT_2 * std::f64::consts::FRAC_2_SQRT_PI;

fn calc_b2b_arm(coef: f64, diff: f64, s2: f64, sqrt_2s2: f64) -> Real {
    let arg = coef / 2.0 * (coef * s2 - 2.0 * diff);
    let arg = num::clamp(arg, MIN_EXP, MAX_EXP); // clip to avoid overflow/underflow
    Real::exp(arg) * libm::erfc((coef * s2 - diff) / sqrt_2s2)
}

pub(crate) struct Back2BackParams<'a> {
    i: Real,
    a: Real,
    b: Real,
    x0: Real,
    s: Real,
    phantom: PhantomData<&'a ()>,
}

struct B2BJacArm {
    exp: f64,
    erfc: f64,
    g: f64,
}

struct CalcArmCoefValueConsts {
    s2: f64,
    sqrt_2s2: f64,
}

fn calc_b2b_jac_arm(consts: &CalcArmCoefValueConsts, arm_coef: f64, diff_x: f64) -> B2BJacArm {
    let arg = 0.5 * arm_coef * (arm_coef * consts.s2 + 2.0 * diff_x);
    let arg = num::clamp(arg, MIN_EXP, MAX_EXP); // clip to avoid overflow/underflow
    let beta = (arm_coef * consts.s2 + diff_x) / consts.sqrt_2s2;
    let exp = f64::exp(arg);
    let erfc = libm::erfc(beta);
    let arg_g = num::clamp(-beta*beta, MIN_EXP, MAX_EXP); // clip to avoid overflow/underflow
    let g = f64::exp(arg_g);
    B2BJacArm { exp, erfc, g }
}

struct CalcDArmCoefConsts {
    i: f64,
    s: f64,
    s2: f64,
    norm_factor: f64,
}

fn calc_d_arm_coef(consts: &CalcDArmCoefConsts, arm_coef: f64, arm: &B2BJacArm, other_arm_coef: f64, diff_x: f64, total_exp_erfc: f64) -> f64 {
    let term_erfc = (arm_coef * consts.s2) / 2.0
        + 0.5 * (arm_coef * consts.s2 + 2.0 * diff_x);
    let temp = total_exp_erfc / (arm_coef + other_arm_coef);
    consts.i * (
        other_arm_coef * temp / 2.0
        + consts.norm_factor * (
            arm.exp * (
                term_erfc * arm.erfc
                - arm.g * FRAC_SQRT_2_SQRT_PI * consts.s
            )
            - temp
        )
    )
}

impl<'a> Accumulator for Back2BackParams<'a> {
    type Jacobian = &'a mut [&'a mut [Real]];

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
        let calc_d_arm_coef_consts = CalcDArmCoefConsts { i: self.i, s: self.s, s2, norm_factor};
        let calc_arm_coef_value_const = CalcArmCoefValueConsts { s2, sqrt_2s2 };

        for (i,x) in input.iter().enumerate() {
            let diff_x = x - self.x0;
            
            // Arm 1 (left)
            let arm_a = calc_b2b_jac_arm(&calc_arm_coef_value_const, self.a, diff_x);
            let arm_b = calc_b2b_jac_arm(&calc_arm_coef_value_const, self.b, diff_x);

            // Common factor
            let total_exp_erfc = arm_a.exp * arm_a.erfc + arm_b.exp * arm_b.erfc;

            // df/dI
            let di = total_exp_erfc * norm_factor;

            // df/da
            let da = calc_d_arm_coef(&calc_d_arm_coef_consts, self.a, &arm_a, self.b, diff_x, total_exp_erfc);

            // df/db
            let db = calc_d_arm_coef(&calc_d_arm_coef_consts, self.b, &arm_b, self.a, diff_x, total_exp_erfc);

            // df/dx0
            let dx0 = self.i * norm_factor
                        * (- self.a * arm_a.exp * arm_a.erfc + self.b * arm_b.exp * arm_b.erfc
                            + FRAC_SQRT_2_SQRT_PI * (arm_a.exp * arm_a.g / self.s - arm_b.exp * arm_b.g / self.s) );

            // df/ds
            let term1 = -2.0 * arm_a.exp * arm_a.g * (std::f64::consts::SQRT_2 * self.a - (self.a * s2 + diff_x) / (std::f64::consts::SQRT_2 * s2)) * std::f64::consts::FRAC_2_SQRT_PI / 2.0;
            let term2 = -2.0 * arm_b.exp * arm_b.g * (std::f64::consts::SQRT_2 * self.b - (self.b * s2 - diff_x) / (std::f64::consts::SQRT_2 * s2)) * std::f64::consts::FRAC_2_SQRT_PI / 2.0;
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

impl<'a> Model for Back2BackParams<'a> {
    type Context = ();
    type Parameters = [Real; 5];
    
    fn init_parameters(_context: & Self::Context) -> Self::Parameters {
        [0.0, 0.0, 0.0, 0.0, 0.0]
    }

    fn new(source: &[Real]) -> Self {
        assert_eq!(source.len(), 5);
        Self {
            i: source[0],
            a: source[1],
            b: source[2],
            x0: source[3],
            s: source[4],
            phantom: PhantomData
        }
    }
}