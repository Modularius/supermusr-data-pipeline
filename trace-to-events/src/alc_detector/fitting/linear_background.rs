use std::marker::PhantomData;

use crate::{alc_detector::fitting::{Accumulator, Model}, pulse_detection::Real};

pub(crate) struct LinearBackground<'a> {
    m: Real,
    c: Real,
    phantom: PhantomData<&'a ()>
}

impl<'a> Accumulator for LinearBackground<'a> {
    type Jacobian = &'a mut [&'a mut [Real]];

    fn accumulate_value(&self, input: &[Real], output: &mut [Real]) {
        for (i,x) in input.iter().enumerate() {
            output[i] = self.m*x + self.c;
        }
    }

    fn accumulate_jacobian(&self, input: &[Real], output: &mut [&mut [Real]]) {
        for (i,x) in input.iter().enumerate() {
            output[i][0] = *x;
            output[i][1] = 1.0;
        }
    }
}

impl<'a> Model for LinearBackground<'a> {
    type Context = ();
    type Parameters = [Real; 2];
    
    fn init_parameters(_context: & Self::Context) -> Self::Parameters {
        [0.0, 0.0]
    }
    
    fn new(source: &[Real]) -> Self {
        assert_eq!(source.len(), 2);
        Self {
            m: source[0],
            c: source[1],
            phantom: PhantomData,
        }
    }
}