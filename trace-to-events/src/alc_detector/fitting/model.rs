use crate::pulse_detection::Real;

/// A value of `dr_i/dp_{j,k}`, where `r_i` is the ith residual,
/// and `p_{j,k}` is the kth parameter of the jth block.
pub(crate) type Derivative = Real;
/// The vector `(dr_i/dp_{j,1},...,dr_i/dp_{j,n_j})`, where `r_i` is the ith residual,
/// and `p_{j,k}` is the kth parameter of the jth block.
pub(crate) type Gradient<'a> = &'a mut [Derivative];
/// The matrix `[... dr_i/dp_{j,k} ...]`, where `r_i` is the ith residual,
/// and `p_{j,k}` is the kth parameter of the jth block.
pub(crate) type Jacobian<'a> = &'a mut [Gradient<'a>];


pub(crate) trait Accumulator {
    fn accumulate_value(&self, input: &[Real], output: &mut [Real]);
    fn accumulate_jacobian(&self, input: &[Real], output: &mut [&mut [Real]]);
}

pub(crate) trait Model : Accumulator {
    type Context;

    fn init_parameters(context: & Self::Context) -> Vec<Real>;
    fn lower_bounds(context: &Self::Context) -> Vec<Option<f64>>;
    fn upper_bounds(context: &Self::Context) -> Vec<Option<f64>>;
    fn new(source: &[Real]) -> Self;
}
