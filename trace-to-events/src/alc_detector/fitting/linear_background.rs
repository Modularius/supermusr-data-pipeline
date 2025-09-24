use crate::{alc_detector::fitting::{Accumulator, Model}, pulse_detection::Real};

pub(crate) struct LinearBackgroundParams {
    m: Real,
    c: Real,
}

pub(crate) struct LinearBackground(LinearBackgroundParams);

impl Accumulator for LinearBackground {
    fn accumulate_value(&self, input: &[Real], output: &mut [Real]) {
        for (i,x) in input.iter().enumerate() {
            output[i] = self.0.m*x + self.0.c;
        }
    }

    fn accumulate_jacobian(&self, input: &[Real], output: &mut [&mut [Real]]) {
        for (i,x) in input.iter().enumerate() {
            output[i][0] = *x;
            output[i][1] = 1.0;
        }
    }
}

impl<'a> Model for LinearBackgroundParams {
    type Context = ();
    type Accumulator = LinearBackground;

    fn accumulator(self) -> Self::Accumulator {
        LinearBackground(self)
    }
    
    fn init_parameters(_context: & Self::Context) -> Vec<Real> {
        vec![0.0; 2]
    }
    
    fn new(source: &[Real]) -> Self {
        assert_eq!(source.len(), 2);
        Self {
            m: source[0],
            c: source[1],
        }
    }
    
    fn lower_bounds(_context: & Self::Context) -> Vec<Option<f64>> {
        vec![None;2]
    }
    
    fn upper_bounds(_context: & Self::Context) -> Vec<Option<f64>> {
        vec![None;2]
    }
}