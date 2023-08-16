use std::collections::VecDeque;
use std::f64::consts::PI;
use std::fmt::Display;

use chrono::Local;

use crate::change_detector::{ChangeData, ChangeClass};
use crate::events::{
    EventData,
    Event,
};
use crate::peak_detector::{PeakData};
use crate::tracedata::Stats;
use crate::trace_iterators::feedback::OptFeedParam;
use crate::{Detector, Real};

use super::FeedbackDetector;


#[derive(Default, Debug, Clone)]
pub struct PulseData {
    peak_time: Option<Real>,
    amplitude: Option<Real>,
    standard_deviation: Option<Real>,
    amplitude_uncertainty: Option<Real>,
    peak_time_uncertainty: Option<Real>,
    effective_interval: Option<(Real,Real)>,
    cache : Vec<Real>,
}
impl PulseData{
    pub fn new(
        peak_time: Option<Real>,
        amplitude: Option<Real>,
        standard_deviation: Option<Real>,
        amplitude_uncertainty: Option<Real>,
        peak_time_uncertainty: Option<Real>,
        effective_interval: Option<(Real,Real)>
    ) -> Self {
        PulseData {
            peak_time, amplitude, standard_deviation,
            amplitude_uncertainty, peak_time_uncertainty,
            effective_interval,
            ..Default::default()
        }
    }
    pub fn new_basic(
        peak_time: Real,
        amplitude: Real,
    ) -> Self {
        PulseData {
            peak_time: Some(peak_time), amplitude: Some(amplitude),
            ..Default::default()
        }
    }
    pub fn with_cache(
        peak_time: Option<Real>,
        amplitude: Option<Real>,
        standard_deviation: Option<Real>,
        amplitude_uncertainty: Option<Real>,
        peak_time_uncertainty: Option<Real>,
        effective_interval: Option<(Real,Real)>
    ) -> Self {
        let pd = PulseData {
            peak_time, amplitude, standard_deviation,
            amplitude_uncertainty, peak_time_uncertainty,
            effective_interval,
            ..Default::default()
        };
        //pd.build_cache();
        pd
    }
    pub fn get_effective_value_at(&self, time : Real) -> Real {
        if self.is_effective_nonzero_at(time) {
            if self.cache.is_empty() {
                self.get_value_at(time)
            } else {
                self.cache[(time - Real::ceil(self.effective_interval.unwrap_or_default().0)) as usize]
            }
        } else {
            Real::default()
        }
    }
    fn build_cache(&mut self) {
        if let Some((start,end)) = self.effective_interval {
            self.cache = Vec::<f64>::with_capacity(Real::ceil(end - start) as usize);
            for i in 0..Real::ceil(end - start) as usize {
                self.cache.push(self.get_value_at(Real::ceil(start) + i as Real));
            }
        }
    }
    pub fn is_effective_nonzero_at(&self, t : Real) -> bool {
        self.effective_interval
            .map(|eff_intv|
                eff_intv.0 <= t && t <= eff_intv.1
            ).unwrap_or(true)
    }
    pub fn get_value_at(&self, t : Real) -> Real {
        self.amplitude.unwrap_or_default() * (-0.5*((t - self.peak_time.unwrap_or_default())/self.standard_deviation.unwrap_or(1.)).powi(2)).exp()
    }
    pub fn get_deriv_at(&self, t : Real) -> Real {
        self.get_value_at(t)*(t - self.peak_time.unwrap_or_default())/self.standard_deviation.unwrap_or(1.).powi(2)
    }
    pub fn get_second_deriv_at(&self, t : Real) -> Real {
        self.get_value_at(t)*(((t - self.peak_time.unwrap_or_default())/self.standard_deviation.unwrap_or(1.)).powi(2) - 1.)/self.standard_deviation.unwrap_or(1.).powi(2)
    }
    pub fn get_peak_time(&self) -> Option<Real> { self.peak_time }
    pub fn get_peak_intensity(&self) -> Option<Real> { self.amplitude }
    pub fn get_standard_deviation(&self) -> Option<Real> { self.standard_deviation }

    pub fn set_peak_time(&mut self, peak_time: Option<Real>) { self.peak_time = peak_time; }
    pub fn set_peak_intensity(&mut self, peak_intensity: Option<Real>) { self.amplitude = peak_intensity; }
    pub fn set_standard_deviation(&mut self, standard_deviation: Option<Real> ) { self.standard_deviation = standard_deviation; }
}

impl EventData for PulseData {}
impl Display for PulseData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0},{1}", self.amplitude.unwrap_or_default(), self.standard_deviation.unwrap_or_default()))
    }
}
pub type PulseEvent = Event<PulseData>;

impl From<Event<PeakData>> for PulseEvent {
    fn from(value: Event<PeakData>) -> Self {
        PulseData::new_basic(value.time, value.data.get_value().unwrap_or_default())
            .make_event(value.time)
    }
}









#[derive(Clone)]
pub struct PulseDetector<D> where D : Detector
{
    detector: D,
    area_under_curve: Real,
    prev_pulses : VecDeque<PulseData>,
    bound : Real,
}

impl<D> PulseDetector<D> where D : Detector {
    pub fn new(detector: D, bound: Real) -> PulseDetector<D> {
        PulseDetector {
            detector,
            area_under_curve: Real::default(),
            prev_pulses: VecDeque::<PulseData>::default(),
            bound
        }
    }

    fn remove_distant_pulses(&mut self, time : Real) {
        loop {
            if let Some(pulse) = self.prev_pulses.front() {
                if pulse.is_effective_nonzero_at(time) {
                    break;
                }
                // LOG
                //log::info!("Old pulse removed from cache: {0:?}",pulse);
                self.prev_pulses.pop_front();
            } else {
                break;
            }
        }
    }

    fn estimate_standard_deviation(&self, amplitude: Real) -> Real {
        2.*self.area_under_curve/Real::sqrt(2.* PI)/amplitude
    }
}

impl<D> Detector for PulseDetector<D> where D : Detector<TimeType = Real, ValueType = Real, DataType = ChangeData> {
    type TimeType = Real;
    type ValueType = Stats;
    type DataType = PulseData;

    fn signal(&mut self, time: Real, value: Self::ValueType) -> Option<PulseEvent> {
        self.area_under_curve += value.mean;
        let _event = self.detector.signal(time, value.mean)?;
        if _event.get_data().get_class() == ChangeClass::Rising {
            self.area_under_curve = 0.;
            return None;
        }
        if _event.get_data().get_class() == ChangeClass::Flat {
            return None;
        }
        let sigma = self.estimate_standard_deviation(value.mean);
        self.area_under_curve = 0.;
        let data = PulseData::with_cache(Some(time),Some(value.mean),Some(sigma),
            Some(value.variance.sqrt()), None,
            Some((time - self.bound*sigma,time + self.bound*sigma)),
        );
        // LOG
        //log::info!("{time}: Pulse data created {data} with window {0}",3.*sigma);
        self.prev_pulses.push_back(data.clone());
        self.area_under_curve = 0.;
        Some(data.make_event(time))
    }
}
impl<D> FeedbackDetector for PulseDetector<D> where D : Detector<TimeType = Real, ValueType = Real, DataType = ChangeData> {
    type ParameterType = Real;

    fn modify_parameter(&mut self, time : Real, param : OptFeedParam<Self::ParameterType>) {
        self.remove_distant_pulses(time);
        //let r = Rc::strong_count(&param.clone().unwrap().0);
        // LOG
        //log::info!("Number of references: {0:?}",r);
        let val = self.prev_pulses.iter().map(|pulse|pulse.get_effective_value_at(time + 1.)).sum::<Real>();
        // LOG
        //log::info!("New correction calculated: {val:?} from {0} pulses", self.prev_pulses.len());
        param.unwrap().set(-val);
    }
}









