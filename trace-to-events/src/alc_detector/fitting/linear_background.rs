use crate::{alc_detector::fitting::Model, pulse_detection::Real};

pub(crate) struct LinearBackground {
    m: Real,
    c: Real,
}

impl Model<2> for LinearBackground {

    fn accumulate_value(&self, input: &[Real], output: &mut [Real]) {
        for (i,x) in input.iter().enumerate() {
            output[i] = self.m*x + self.c;
        }
    }

    fn accumulate_jacobian(&self, input: &[Real], output: &mut [[Real; 2]]) {
        for (i,x) in input.iter().enumerate() {
            output[i] = [*x, 1.0];
        }
    }
}