use std::array::{from_fn, from_ref};
use std::fmt::Display;

use crate::events::{
    EventData,
    event::Event,
};
use crate::window::Window;
use crate::{Detector, Real, RealArray, SmoothingWindow};

#[derive(Default, Debug, Clone, PartialEq)]
pub enum ChangeClass {
    #[default]
    Flat,
    Rising,
    Falling,
}

#[derive(Default, Debug, Clone)]
pub struct ChangeData {
    pub(super) class: ChangeClass,
    value : Real,
}
impl EventData for ChangeData {}
pub type ChangeEvent = Event<ChangeData>;

impl Display for ChangeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0},{1}",
        self.value,
            match self.class {
                ChangeClass::Rising => 1i32,
                ChangeClass::Flat => 0i32,
                ChangeClass::Falling => -1i32,
            }
        ))
    }
}

#[derive(Default,Clone)]
pub struct ChangeDetector {
    mode: ChangeClass,
    prev: Option<Real>,
    threshold: Real,
}
impl ChangeDetector {
    pub fn new(threshold: Real) -> Self {
        Self {
            threshold,
            ..Default::default()
        }
    }
}
impl Detector for ChangeDetector {
    type TimeType = Real;
    type ValueType = Real;
    type DataType = ChangeData;

    fn signal(&mut self, time: Real, value: Real) -> Option<ChangeEvent> {
        if let Some(prev_value) = self.prev {
            let new_mode = {
                if (value - prev_value).abs() <= self.threshold {
                    ChangeClass::Flat
                } else if value > prev_value {
                    ChangeClass::Rising
                } else {
                    ChangeClass::Falling
                }
            };

            let event_class = if new_mode == self.mode {
                None
            } else {
                Some(new_mode.clone())
            };
            self.mode = new_mode;
            self.prev = Some(value);
            event_class.map(|e| ChangeEvent::new(time, ChangeData { class: e.clone(), value }))
        } else {
            self.prev = Some(value);
            None
        }
    }
}