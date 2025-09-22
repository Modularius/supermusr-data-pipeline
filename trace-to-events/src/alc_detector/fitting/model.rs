use crate::pulse_detection::Real;


pub(crate) trait Accumulator {
    type Jacobian;

    fn accumulate_value(&self, input: &[Real], output: &mut [Real]);
    fn accumulate_jacobian(&self, input: &[Real], output: Self::Jacobian);
}

pub(crate) trait Model : Accumulator{
    type Context;
    type Parameters;

    fn init_parameters(context: & Self::Context) -> Self::Parameters;
    fn new(source: &[Real]) -> Self;
}
