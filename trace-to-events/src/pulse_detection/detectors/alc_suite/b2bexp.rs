////*
/// * Back to Back Exponential, its gradient and sums of this function
///* See https://docs.mantidproject.org/nightly/fitting/fitfunctions/BackToBackExponential.html
///*
///* Copyright (C) 2025 The Science and Technology Facilities Council (STFC)
///* Authors: Boris Shustin (STFC) and Jaroslav Fowkes (STFC)
///*/

/*
 * An implementation of the linear background function
 *
 * Inputs:
 *
 * nx - number of evaluation points
 * x - 1D array of evaluation points
 * params - 1D array of the two function parameters:
 *          m - slope of the linear background
 *          c - intercept of the linear background
 * acc_out - zero-initialised 1D array to accumulate function values (nx)
 *
 * Outputs:
 *
 * acc_out - 1D array of accumulated function values at evaluation points (nx)
 *
 */
pub(super) struct LinParams {
    m: f64,
    c: f64
}

pub(super) fn linear_background(x: &[f64], params: LinParams, mut acc_out: Vec<f64>) -> Vec<f64> {
    for (i,x) in x.iter().enumerate() {
        acc_out[i] += params.m * x + params.c;
    }
    acc_out
}

/*
 * Jacobian of the linear background function
 *
 * Inputs:
 *
 * nx - number of evaluation points
 * x - 1D array of evaluation points
 * jac - 1D array to hold jacobian values at evaluation points (nx*2)
 *
 * Outputs:
 *
 * jac - 1D array of jacobian values at evaluation points (nx*2)
 *
 */
pub(super) fn linear_background_jacobian(x: &[f64], mut jac: Vec<f64>) -> Vec<f64> {
    for (i,x) in x.iter().enumerate() {
        jac[i*2 + 0] = *x;
        jac[i*2 + 1] = 1.0;
    }
    jac
}

/*
 * An implementation of the back-to-back exponential peak function
 *
 * Inputs:
 *
 * nx - number of evaluation points
 * x - 1D array of evaluation points
 * params - 1D array of the five function parameters:
 *          I - integrated intensity of the peak
 *          a - exponential constant of rising part
 *          b - exponential constant of decaying part
 *          x0 - peak position
 *          s - standard deviation of gaussian part of peakshape function
 * acc_out - zero-initialised 1D array to accumulate function values (nx)
 *
 * Outputs:
 *
 * acc_out - 1D array of accumulated function values at evaluation points (nx)
 *
 */
pub(super) struct B2BParams {
    I: f64,
    a: f64,
    b: f64,
    x0: f64,
    s: f64,
}

pub(super) fn back_to_back_exponential(x: &[f64], params: &B2BParams, mut acc_out: Vec<f64>) -> Vec<f64> {
    let s2 = params.s * params.s;
    let sqrt_2s2 = f64::sqrt(2.0) * params.s; // Should this be s2?

    let norm_factor = params.a * params.b / (2.0 * (params.a + params.b));

    for (i,val) in x.iter().map(|x| {
        let diff = x - params.x0;
        let arg1 = params.a / 2.0 * (params.a * s2 + 2.0 * diff);
        let val1 = f64::exp(arg1) * f64::erfc((params.a * s2 + diff) / sqrt_2s2);
        let arg2 = params.b / 2.0 * (params.b * s2 - 2.0 * diff);
        let val2 = f64::exp(arg2) * f64::erfc((params.b * s2 - diff) / sqrt_2s2);
        params.I * (val1 + val2) * norm_factor
    }).enumerate() {
        acc_out[i] += val;
    }
    acc_out
}

/*
 * Jacobian of the back-to-back exponential peak function
 *
 * Inputs:
 *
 * nx - number of evaluation points
 * x - 1D array of evaluation points
 * params - 1D array of the five function parameters:
 *          I - integrated intensity of the peak
 *          a - exponential constant of rising part
 *          b - exponential constant of decaying part
 *          x0 - peak position
 *          s - standard deviation of gaussian part of peakshape function
 * jac - 1D array to hold jacobian values at evaluation points (nx*5)
 *
 * Outputs:
 *
 * jac - 1D array of jacobian values at evaluation points (nx*5)
 *
 */
pub(super) fn back_to_back_jacobian(x: &[f64], params: &B2BParams, jac: Vec<f64>) -> Vec<f64> {
    let s2 = params.s * params.s;
    let sqrt_2s2 = f64::sqrt(2.0) * params.s; // Should this be s2?

    let norm_factor = params.a * params.b / (2.0 * (params.a + params.b));

    for (i,(dI,da,db,dx0,ds)) in x.iter().map(|x| {
        let diff = x - x0;

        // Arm 1 (left)
        let arg1 = 0.5 * params.a * (params.a * s2 + 2.0 * diff);
        let beta1 = (params.a * s2 + diff) / sqrt_2s2;
        let exp1 = f64::exp(arg1);
        let erfc1 = f64::erfc(beta1);
        let G1 = f64::exp(-beta1*beta1);

        // Arm 2 (right)
        let arg2 = 0.5 * params.b * (params.b * s2 - 2.0 * diff);
        let beta2 = (params.b * s2 - diff) / sqrt_2s2;
        let exp2 = f64::exp(arg2);
        let erfc2 = f64::erfc(beta2);
        let G2 = f64::exp(-beta2*beta2);

        // Common factor
        let total_exp_erfc = exp1 * erfc1 + exp2 * erfc2;

        // df/dI
        let dI = total_exp_erfc * norm_factor;

        // df/da
        let terma_erfc = (params.a * s2) / 2.0 + 0.5 * (params.a * s2 + 2.0 * diff);
        let da = - params.I * norm_factor * total_exp_erfc / (params.a + params.b)
                 + params.I * params.b * total_exp_erfc / (2.0 * (params.a + params.b))
                 + params.I * norm_factor
                 * (- exp1 * G1 * f64::sqrt(2.0 / M_PI) * params.s
                    + exp1 * terma_erfc * erfc1);

        // df/db
        let termb_erfc = (params.b * s2) / 2.0 + 0.5 * (params.b * s2 - 2.0 * diff);
        let db = - params.I * norm_factor * total_exp_erfc / (params.a + params.b)
                    + params.I * params.a * total_exp_erfc / (2.0 * (params.a + params.b))
                    + params.I * norm_factor
                    * (- exp2 * G2 * f64::sqrt(2.0 / M_PI) * params.s
                       + exp2 * termb_erfc * erfc2);

        // df/dx0
        let dx0 = params.I * norm_factor
                       * (- params.a * exp1 * erfc1 + params.b * exp2 * erfc2
                          + f64::sqrt(2.0 / M_PI) * (exp1 * G1 / params.s - exp2 * G2 / params.s) );

        // df/ds
        let term1 = -2.0 * exp1 * G1 * (f64::sqrt(2.0) * params.a - (params.a * s2 + diff) / (f64::sqrt(2.0) * s2)) / f64::sqrt(M_PI);
        let term2 = -2.0 * exp2 * G2 * (f64::sqrt(2.0) * params.b - (params.b * s2 - diff) / (f64::sqrt(2.0) * s2)) / f64::sqrt(M_PI);
        let term3 = params.a * params.a * params.s * exp1 * erfc1;
        let term4 = params.b * params.b * params.s * exp2 * erfc2;
        let ds = params.I * norm_factor * (term1 + term2 + term3 + term4);

        (dI,da,db,dx0,ds)
    }).enumerate() {
        // Jacobian
        jac[i*5 + 0] = dI;
        jac[i*5 + 1] = da;
        jac[i*5 + 2] = db;
        jac[i*5 + 3] = dx0;
        jac[i*5 + 4] = ds;
    }
    jac
}

/*
 * Sum of back-to-back exponential peak function residuals with a linear background
 *
 * Inputs:
 *
 * nx - number of evaluation points
 * x - 1D array of evaluation points
 * y - 1D array of function values at evaluation points
 * npeaks - number of peaks
 * params - array of arrays of function parameters for all peaks and linear background
 *          the first npeak b2b peak arrays are of size 5
 *          the final linear background array is of size 2
 * res - 1D array to hold residual values at evaluation points (nx)
 *
 * Outputs:
 *
 * res - 1D array of residual values at evaluation points (nx)
 *
 */
pub(super) fn sum_of_back_to_back_residuals(x: &[f64], y: &[f64], params: &[B2BParams]) -> Vec<f64> {
                                    
    // res[i] = sum_k b2b_k(x[i]) + m*x[i] + c - y[i]
    // initialise accumulator res with negative true values
    let res = y.iter().map(|y|-y).collect::<Vec<_>>();

    // accumulate b2bexp values into res
    for k in 0..params.len() {
        res = back_to_back_exponential(x, &params[k], res)
    }
    // accumulate linear background values into res
    linear_background(x, params[npeaks], res)
}

/*
 * Jacobian of the sum of back-to-back exponential peak functions with a linear background
 *
 * Inputs:
 *
 * nx - number of evaluation points
 * x - 1D array of evaluation points
 * npeaks - number of peaks
 * params - array of arrays of function parameters for all peaks and linear background
 *          the first npeak b2b peak arrays are of size 5
 *          the final linear background array is of size 2
 * jac - array of arrays to hold jacobian values
 *       the first npeak b2b peak arrays are of size nx*5
 *       the final linear background array is of size nx*2
 *
 * Outputs:
 *
 * jac - array of arrays of jacobian at evaluation points
 *       the first npeak b2b peak arrays are of size nx*5
 *       the final linear background array is of size nx*2
 *
 */
pub(super) fn sum_of_back_to_back_jacobian(x: &[f64], params: &[B2BParams], mut jac: Vec<Vec<f64>>) {

    // write b2bexp jacobians to jac
    for (k,params) in params.iter().enumerate() {
        *jac.get_mut(k).expect("") = back_to_back_jacobian(x, params, jac.pop().expect(""));
    }
    // write linear background jacobians to jac
    linear_background_jacobian(x, jac.pop().expect(""));

}