use std::fmt::Display;

use crate::events::{
    EventData, 
    event::Event,
};
use crate::{Detector, Real};

#[derive(Default, Debug, Clone,PartialEq)]
pub enum PeakClass {
    #[default]
    Flat,
    LocalMax,
    LocalMin,
}

#[derive(Default, Debug, Clone)]
pub struct PeakData {
    class: PeakClass,
    value : Option<Real>,
}
impl PeakData {
    pub fn get_class(&self) -> PeakClass { self.class.clone() }
    pub fn get_value(&self) -> Option<Real> { self.value }
}
impl EventData for PeakData {}

impl Display for PeakData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{0},{1}", self.value.unwrap_or(0.),
            match self.class { PeakClass::LocalMax => 1, PeakClass::Flat => 0, PeakClass::LocalMin => -1, }
        ))
    }
}

type PeakEvent = Event<PeakData>;

#[derive(Default)]
pub struct PeakDetector {
    prev: Option<(Real,Option<Real>)>,
}

impl PeakDetector {
    pub fn new() -> Self { Self::default() }
}

impl Detector for PeakDetector {
    type TimeType = Real;
    type ValueType = Real;
    type DataType = PeakData;

    fn signal(&mut self, time: Real, value: Real) -> Option<PeakEvent> {
        if let Some((prev_value,Some(prev_prev_value))) = self.prev {
            let return_value = {
                if (prev_prev_value < prev_value && prev_value >= value) || (prev_prev_value <= prev_value && prev_value > value) {
                    Some(PeakEvent::new(
                        time - 1.,
                        PeakData {
                            class: PeakClass::LocalMax,
                            value: Some(prev_value),
                        },
                    ))
                } else if (prev_prev_value > prev_value && prev_value <= value) || (prev_prev_value >= prev_value && prev_value < value) {
                    Some(PeakEvent::new(
                        time - 1.,
                        PeakData {
                            class: PeakClass::LocalMin,
                            value: Some(prev_value),
                        },
                    ))
                } else { None }
            };
            self.prev = Some((value,Some(prev_value)));
            return_value       
        } else if let Some((prev_value,None)) = self.prev {
            self.prev = Some((value,Some(prev_value)));
            None
        }
        else {
            self.prev = Some((value,None));
            None
        }
    }
}
