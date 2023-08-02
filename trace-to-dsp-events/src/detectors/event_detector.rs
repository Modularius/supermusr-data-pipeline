use std::collections::VecDeque;
use std::fmt::Display;

use crate::events::{
    Event,
    EventData,
    EventWithData,
    SimpleEvent,
    TimeValue
};
use crate::window::{Window, smoothing_window};
use crate::window::smoothing_window::{SmoothingWindow,SNRSign, Stats};
use crate::{Detector, Real, RealArray};
use common::Intensity;
use fitting::approx::assert_abs_diff_eq;
use fitting::Gaussian;
use fitting::gaussian::GaussianError;
use fitting::ndarray::{array, Array, Array1};
use num::complex::ComplexFloat;

use super::change_detector;
use super::composite::CompositeDetector;

#[derive(Default, Debug, Clone)]
pub enum Class {
    #[default]
    Flat,
    Rising,
    Falling,
    LocalMax,
    LocalMin,
}
#[derive(Default, Debug, Clone)]
pub struct Data {
    pub(super) class: Class,
    peak_intensity: Option<Real>,
    area_under_curve: Option<Real>,
    half_peak_full_width: Option<Real>,
    start: Option<Real>,
    end: Option<Real>,
}

impl EventData for Data {}
impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{0}",
            match self.class {
                Class::Rising => 1i32,
                Class::Flat => 0i32,
                Class::Falling => -1i32,
                Class::LocalMax => self.peak_intensity.unwrap_or_default() as i32,
                Class::LocalMin => -(self.peak_intensity.unwrap_or_default() as i32),
            }
        ))
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
enum SignalState {
    #[default]
    Flat,
    High,
    Low,
}
impl SignalState {
    fn from_stats(stats: &Stats, threshold: Real) -> Option<(Self, Real)> {
        if stats.variance == 0. {
            return None;
        }
        match stats.signal_over_noise_sign(threshold) {
            SNRSign::Pos => Some((SignalState::High, stats.get_normalized_value())),
            SNRSign::Neg => Some((SignalState::Low, stats.get_normalized_value())),
            SNRSign::Zero => Some((SignalState::Flat, 0.)),
        }
    }
}

enum EventsDetectorState {
    WaitingForNonzero,
    WaitingForChange,
}
pub struct EventsDetector<const N: usize> {
    state : EventsDetectorState,

    base_line : Real,

    prev_signal: Vec<(Real, Real)>,
    pulses_in_progress: VecDeque<Gaussian<Real>>,

    change_detector: CompositeDetector<N, SimpleEvent<change_detector::Data>>,

    baseline_detector : SmoothingWindow,
}

impl<const N: usize> EventsDetector<N> {
    pub fn new(
        change_detector: CompositeDetector<N, SimpleEvent<change_detector::Data>>,
        baseline_detector: SmoothingWindow,
    ) -> EventsDetector<N> {
        EventsDetector {
            change_detector,
            state: EventsDetectorState::WaitingForNonzero,
            base_line : Real::default(),
            prev_signal: Vec::<(Real, Real)>::default(),
            pulses_in_progress: VecDeque::<Gaussian<Real>>::default(),
            baseline_detector,
        }
    }
    fn extract_gaussian(&mut self) -> Result<Gaussian<Real>,GaussianError> {
        let x_vec : Array1<Real> = self.prev_signal.iter().map(|s|s.0).collect();
        let y_vec : Array1<Real> = self.prev_signal.iter().map(|s|s.1).collect();
        println!("{x_vec:?}, {y_vec:?}");
        let estimated = Gaussian::fit(x_vec.clone(), y_vec.clone())?;

        self.prev_signal.clear();
        Ok(estimated)
    }
    fn correct_signal(&mut self, time : Real, value : Real) -> (Real,Real) {
        (time,Real::min(value - self.base_line - self.pulses_in_progress.iter().map(|g|g.value(time)).sum::<Real>(),0.))
    }
    fn detect_event(&mut self, time: Real, value: RealArray<N>) -> Option<change_detector::Data>{
        if let Some(events)= self.change_detector.signal(time, value) {
            let mut iter = events.into_iter();
            if let Some(event) = iter.find(|e| e.get_data().get_index() == 1) {
                return Some(event.get_data().get_data().clone())
            }
        }
        None
    }
    fn detect_next_pulse(&mut self, time: Real, value: RealArray<N>) {
        let new_gaussian = match self.extract_gaussian() {
            Ok(gaussian) => gaussian,
            Err(e) => panic!("{:?}\n{e}",self.pulses_in_progress),
        };
        self.pulses_in_progress.push_back(new_gaussian);
    }
}

impl<const N: usize> Detector for EventsDetector<N> {
    type TimeType = Real;
    type ValueType = RealArray<N>;
    type EventType = SimpleEvent<Data>;

    fn signal(&mut self, time: Real, value: Self::ValueType) -> Option<SimpleEvent<Data>> {
        match &self.state {
            EventsDetectorState::WaitingForNonzero => {
                self.baseline_detector.push(value[0]);
                if let Some(event) = self.detect_event(time, value) {
                    if change_detector::Class::Flat != event.class {
                        self.state= EventsDetectorState::WaitingForChange;
                    }
                };
            },
            EventsDetectorState::WaitingForChange => {
                if let Some(_) = self.detect_event(time, value) {
                    self.detect_next_pulse(time, value);
                    if let Some(stats) = self.baseline_detector.stats() {
                        let self.base_line = smoothing_window::extract::mean(stats);
                        for (i,val) in self.prev_signal.iter_mut() {
                            *val -= mean;
                        }
                    }
                }
                let signal = self.correct_signal(time, value[0]);
                self.prev_signal.push(signal);
            },
        };
        if let Some(pulse) = self.pulses_in_progress.front() {
            if pulse.value(time).abs() < 1e-6 {
                let pulse = self.pulses_in_progress.pop_front().unwrap();
                return Some(
                    SimpleEvent::<Data>::new(
                        *pulse.mu(),
                         Data {
                             class: Class::Flat,
                             peak_intensity: Some(*pulse.a()),
                             area_under_curve: None,
                             half_peak_full_width: Some(*pulse.sigma()),
                             start: None,
                             end: None
                        }
                    )
                );
            }
        }
        None
    }
}
