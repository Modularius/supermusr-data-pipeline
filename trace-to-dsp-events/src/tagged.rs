use std::fmt::{Display, Formatter, Result, Debug};

use crate::{Real, trace_iterators::TraceData};


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
    pub fn mean<D>(trace: D) -> Real where D : TraceData<ValueType = Stats> {
        trace.get_value().mean
    }
    pub fn enumerated_mean<T,D>(trace: D) -> (T,Real) where D : TraceData<TimeType = T, ValueType = Stats> {
        (trace.get_time(), trace.get_value().mean)
    }
    pub fn enumerated_variance<T,D>(trace: D) -> (T,Real) where D : TraceData<TimeType = T, ValueType = Stats> {
        (trace.get_time(), trace.get_value().variance)
    }
    pub fn enumerated_standard_deviation<T,D>(trace: D) -> (T,Real) where D : TraceData<TimeType = T, ValueType = Stats> {
        (trace.get_time(), trace.get_value().variance.sqrt())
    }
    pub fn enumerated_normalised_mean<T,D>(trace: D) -> (T,Real) where D : TraceData<TimeType = T, ValueType = Stats> {
        if trace.get_value().variance == 0. {
            (trace.get_time(), trace.get_value().mean)
        } else {
            (trace.get_time(), trace.get_value().mean / trace.get_value().variance.sqrt())
        }
    }
    pub fn enumerated_normalised_value<T,D>(trace: D) -> (T,Real) where D : TraceData<TimeType = T, ValueType = Stats> {
        if trace.get_value().variance == 0. {
            (trace.get_time(), trace.get_value().value)
        } else {
            (trace.get_time(), (trace.get_value().value - trace.get_value().mean) / trace.get_value().variance.sqrt())
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
