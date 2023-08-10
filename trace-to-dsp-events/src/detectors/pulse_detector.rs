use std::collections::VecDeque;
use std::f64::consts::PI;
use std::fmt::Display;

use crate::events::{
    EventData,
    Event,
};
use crate::tagged::Stats;
use crate::trace_iterators::feedback::OptFeedParam;
use crate::{Detector, Real};

use super::FeedbackDetector;


#[derive(Default, Debug, Clone)]
pub struct PulseData {
    peak_time: Option<Real>,
    peak_intensity: Option<Real>,
    std_dev: Option<Real>,
}
impl PulseData{
    pub fn get_value_at(&self, t : Real) -> Real {
        self.peak_intensity.unwrap_or_default() * (-0.5*((t - self.peak_time.unwrap_or_default())/self.std_dev.unwrap_or(1.)).powi(2)).exp()
    }
    pub fn get_peak_time(&self) -> Option<Real> { self.peak_time }
    pub fn get_peak_intensity(&self) -> Option<Real> { self.peak_intensity }
    pub fn get_standard_deviation(&self) -> Option<Real> { self.std_dev }

    pub fn set_peak_time(&mut self, peak_time: Option<Real>) { self.peak_time = peak_time; }
    pub fn set_peak_intensity(&mut self, peak_intensity: Option<Real>) { self.peak_intensity = peak_intensity; }
    pub fn set_standard_deviation(&mut self, std_dev: Option<Real> ) { self.std_dev = std_dev; }
}

impl EventData for PulseData {}
impl Display for PulseData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0},{1}", self.peak_intensity.unwrap_or_default(), self.std_dev.unwrap_or_default()))
    }
}
pub type PulseEvent = Event<PulseData>;










pub struct PulseDetector<D> where D : Detector<TimeType = Real, ValueType = Real>
{
    detector: D,
    area_under_curve: Real,
    prev_pulses : VecDeque<PulseData>,
}

impl<D> PulseDetector<D> where D : Detector<TimeType = Real, ValueType = Real> {
    pub fn new(detector: D) -> PulseDetector<D> {
        PulseDetector {
            detector,
            area_under_curve: Real::default(),
            prev_pulses: VecDeque::<PulseData>::default(),
        }
    }
}

impl<D> Detector for PulseDetector<D> where D : Detector<TimeType = Real, ValueType = Real> {
    type TimeType = Real;
    type ValueType = Stats;
    type DataType = PulseData;

    fn signal(&mut self, time: Real, value: Self::ValueType) -> Option<PulseEvent> {
        self.area_under_curve += value.value;
        match self.detector.signal(time, value.value) {
            Some(event) => {
                event.get_data();
                let data = PulseData {
                    peak_time: Some(time),
                    peak_intensity: Some(value.value + value.variance.sqrt()),
                    std_dev: Some(2.*self.area_under_curve/Real::sqrt(2.* PI)),
                };
                self.prev_pulses.push_back(data.clone());
                self.area_under_curve = 0.;
                Some(PulseEvent::new(time, data))
            },
            None => {
                None
            }
        }
    }
}
impl<D> FeedbackDetector for PulseDetector<D> where D : Detector<TimeType = Real, ValueType = Real> {
    type ParameterType = Real;

    fn modify_parameter(&mut self, time : Real, param : OptFeedParam<Self::ParameterType>) {
        let val = self.prev_pulses.iter().map(|pulse|pulse.get_value_at(time)).sum::<Real>();
        param.unwrap().set(-val);
    }
}









