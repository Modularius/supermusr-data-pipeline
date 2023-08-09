use std::fmt::{Display, Formatter, Result};

use crate::{EnumeratedValue, Real};

pub trait ValueWithTaggedData {
    type TaggedType : Default + Clone + Display;

    fn get_value(&self) -> &Real;
    fn get_value_mut(&mut self) -> &mut Real;
    fn get_tagged_data(&self) -> &Self::TaggedType;
}


impl ValueWithTaggedData for Stats {
    type TaggedType = Stats;

    fn get_value(&self) -> &Real { &self.value }
    fn get_value_mut(&mut self) -> &mut Real  { &mut self.value }
    fn get_tagged_data(&self) -> &Stats  { &self }

}

#[derive(Default, Clone, Debug)]
pub struct Stats {
    pub value: Real,
    pub mean: Real,
    pub variance: Real,
}
impl From<Real> for Stats {
    fn from(value: Real) -> Self {
        Stats { value, mean: value, variance: 0. }
    }
}
impl Display for Stats {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "({}:?, {}:?, {}:?)", self.value, self.mean, self.variance)
    }
}


pub mod extract {
    use super::*;
    pub fn mean((i, Stats { value, mean, variance, }): (Real,Stats)) -> Real {
        mean
    }
    pub fn enumerated_mean((i, Stats { value, mean, variance, }): (Real,Stats)) -> (Real,Real) {
        (i, mean)
    }
    pub fn enumerated_variance((i, Stats { value, mean, variance, }): (Real,Stats)) -> EnumeratedValue {
        (i, variance)
    }
    pub fn enumerated_standard_deviation((i, Stats { value, mean, variance, }): (Real,Stats)) -> EnumeratedValue {
        (i, variance.sqrt())
    }
    pub fn enumerated_normalised_mean((i, Stats { value, mean, variance, }): (Real,Stats)) -> EnumeratedValue {
        if variance == 0. {
            (i, mean)
        } else {
            (i, mean / variance.sqrt())
        }
    }
    pub fn enumerated_normalised_value((i, Stats { value, mean, variance, }): (Real,Stats)) -> (Real,Real) {
        if variance == 0. {
            (i, value)
        } else {
            (i, (value - mean) / variance.sqrt() + mean)
        }
    }
}



#[derive(Default, Clone)]
pub enum SNRSign {
    Pos,
    Neg,
    #[default]
    Zero,
}
impl Stats {
    pub fn signal_over_noise_sign(&self, threshold: Real) -> SNRSign {
        if (self.value - self.mean).powi(2) >= self.variance * threshold.powi(2) {
            if (self.value - self.mean).is_sign_positive() {
                SNRSign::Pos
            } else {
                SNRSign::Neg
            }
        } else {
            SNRSign::Zero
        }
    }
    pub fn get_normalized_value(&self) -> Real {
        (self.value - self.mean).powi(2) / self.variance.sqrt()
    }
    pub fn shift(&mut self, delta: Real) {
        self.value += delta;
        self.mean += delta;
    }
}
