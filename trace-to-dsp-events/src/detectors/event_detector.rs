use std::collections::VecDeque;
use std::fmt::Display;

use crate::events::{Event, EventData, EventWithData, SimpleEvent, TimeValue};
use crate::window::smoothing_window::{SNRSign, Stats};
use crate::{Detector, Real, RealArray};
use common::Intensity;

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

pub struct EventsDetector<const N: usize> {
    prev_signal: VecDeque<(Real, RealArray<N>)>,
    pulses_begun: VecDeque<Real>,
    pulses_found: VecDeque<SimpleEvent<Data>>,

    change_detector: CompositeDetector<N, SimpleEvent<change_detector::Data>>,
}

impl<const N: usize> EventsDetector<N> {
    pub fn new(
        change_detector: CompositeDetector<N, SimpleEvent<change_detector::Data>>,
    ) -> EventsDetector<N> {
        EventsDetector {
            change_detector,
            prev_signal: VecDeque::<(Real, RealArray<N>)>::default(),
            pulses_begun: VecDeque::<Real>::default(),
            pulses_found: VecDeque::<SimpleEvent<Data>>::default(),
        }
    }
    fn retro(&mut self) {
        for (time, values) in &mut self.prev_signal {
            for pulse in &mut self.pulses_found {
                for n in 0..N {
                    values[n] -= Self::gaussian_diff(
                        pulse.get_time(),
                        pulse.get_data().half_peak_full_width.unwrap(),
                        pulse.get_data().peak_intensity.unwrap(),
                        *time,
                        n,
                    );
                }
            }
        }
    }
    fn gaussian_diff(mu: Real, sigma2: Real, peak: Real, time: Real, n: usize) -> Real {
        if n == 0 {
            -peak * (0.5 * (time - mu).powi(2) / sigma2).exp()
        } else {
            ((time - mu) * mu / sigma2) * Self::gaussian_diff(mu, sigma2, peak, time, 0)
                - (0.5 * (time - mu).powi(2) / sigma2)
                    * Self::gaussian_diff(mu, sigma2, peak, time, n - 1)
        }
    }
}
impl<const N: usize> Detector for EventsDetector<N> {
    type TimeType = Real;
    type ValueType = RealArray<N>;
    type EventType = SimpleEvent<Data>;

    fn signal(&mut self, time: Real, value: Self::ValueType) -> Option<SimpleEvent<Data>> {
        self.prev_signal.push_back((time, value));
        match self.change_detector.signal(time, value) {
            Some(events) => {
                let mut iter = events.into_iter();
                if let Some(event) = iter.find(|e| e.get_data().get_index() == 1) {
                    match event.get_data().get_data().class {
                        change_detector::Class::Flat => {
                            //self.pulses_found.push_back(time)
                        }
                        change_detector::Class::Rising => self.pulses_begun.push_back(time),
                        change_detector::Class::Falling => todo!(),
                    }
                }
                None
            }
            None => None,
        }
    }
}
