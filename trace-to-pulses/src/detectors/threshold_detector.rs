use std::fmt::Display;

use crate::{Real, Detector};
use crate::events::Event;
use crate::tracedata::EventData;

#[derive(Default, Debug, Clone)]
pub struct ThresholdData {
    pub(super) peak: Option<Real>,
    pub(super) peak_time: Option<Real>,
    pub(super) start: Option<Real>,
    pub(super) end: Option<Real>,
}

impl ThresholdData {
    pub fn new(
        peak: Real,
        peak_time: Real,
        start: Real,
        end: Real,
    ) -> Self {
        Self {
            peak: Some(peak), peak_time: Some(peak_time), start: Some(start), end: Some(end),
            ..Default::default()
        }
    }
}

impl Display for ThresholdData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0},{1},{2}", self.peak.unwrap_or_default(), self.start.unwrap_or_default(), self.end.unwrap_or_default()))
    }
}
impl EventData for ThresholdData {}



#[derive(Default,Clone)]
pub struct ThresholdDetector {
    threshold : Real,
    
    event_in_progress: Option<(Real,Real,Real)>,
}

impl ThresholdDetector {
    pub fn new(threshold : Real) -> Self { Self { threshold, ..Default::default() } }
}

pub type ThresholdEvent = Event<Real, ThresholdData>;

impl Detector for ThresholdDetector {
    type TimeType = Real;
    type ValueType = Real;
    type DataType = ThresholdData;

    fn signal(&mut self, time: Real, value: Real) -> Option<ThresholdEvent> {
        match self.event_in_progress {
            Some((event_start, max_value, max_value_at)) => {
                let (max_value,max_value_at) =
                    if max_value < value { (value, time) }
                    else { (max_value,max_value_at) };
                if value < self.threshold {
                    self.event_in_progress = None;
                    Some(ThresholdData::new(max_value, max_value_at, event_start, time).make_event(time))
                } else {
                    self.event_in_progress = Some((event_start, max_value, max_value_at));
                    None
                }
            },
            None => {
                if value >= self.threshold {
                    self.event_in_progress = Some((time, value, time));
                }
                None
            },
        }
    }
}