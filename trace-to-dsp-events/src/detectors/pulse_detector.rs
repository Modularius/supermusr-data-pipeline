use std::collections::VecDeque;
use std::default;
use std::fmt::Display;

use crate::events::{
    EventData,
    event::Event,
};
use crate::tagged::{ValueWithTaggedData, Stats};
use crate::window::{Window, smoothing_window};
use crate::{Detector, Real, RealArray};
//use fitting::Gaussian;
//use fitting::gaussian::GaussianError;
use fitting::ndarray::{array, Array, Array1};

use log;

use super::change_detector::{
    ChangeDetector,
    ChangeEvent,
};
use super::composite::{CompositeDetector, CompositeData, CompositeEvent};

#[derive(Default, Debug, Clone)]
pub struct PulseData {
    peak_time: Option<Real>,
    peak_intensity: Option<Real>,
    std_dev: Option<Real>,
}

impl EventData for PulseData {}
impl Display for PulseData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(""))
    }
}
pub type PulseEvent = Event<PulseData>;

pub struct PulseDetector<D> where D : Detector<TimeType = Real, ValueType = Stats>
{
    detector: D,
    area_under_curve: Real,
    prev_values: Option<D>,
}

impl<D> PulseDetector<D> where D : Detector<TimeType = Real, ValueType = Stats> {
    pub fn new(detector: D) -> PulseDetector<D> {
        PulseDetector {
            detector,
            area_under_curve: Real::default(),
            prev_values: None,
        }
    }
}

impl<D> Detector for PulseDetector<D> where D : Detector<TimeType = Real, ValueType = Stats> {
    type TimeType = Real;
    type ValueType = Stats;
    type DataType = PulseData;

    fn signal(&mut self, time: Real, value: Self::ValueType) -> Option<PulseEvent> {
        self.area_under_curve += value.get_value();
        match self.detector.signal(time, value) {
            Some(event) => {
                event.get_data();
                self.area_under_curve = 0.;
                return None
            },
            None => {
                None
            }
        }
    }
}









