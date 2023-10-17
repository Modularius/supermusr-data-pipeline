use std::fmt::Display;

use crate::events::Event;
use crate::pulse::{Pulse, TimeValue, TimeValueOptional};
use crate::tracedata::{EventData, Stats, TraceValue};
use crate::{Detector, Real, RealArray};

use super::Assembler;

#[derive(Default, Debug, Clone,PartialEq)]
pub enum Class {
    #[default] Onset,
    Peak,
    End,
    EndOnset,
}
impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Onset => "0",
            Self::Peak => "2",
            Self::End => "-1",
            Self::EndOnset => "-2",
        })
    }
}

#[derive(Default, Debug, Clone)]
pub struct Data {
    class: Class,
    value : Real,
    superlative : Option<TimeValue<RealArray<2>>>,
}
impl Data {
    pub fn new(class : Class, value : Real, superlative : Option<TimeValue<RealArray<2>>>) -> Self {
        Data { class, value, superlative }
    }
    pub fn get_class(&self) -> Class { self.class.clone() }
    pub fn get_value(&self) -> Real { self.value }
    pub fn get_superlative(&self) -> Option<TimeValue<RealArray<2>>> { self.superlative.clone() }
}
impl EventData for Data {}

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0},{1}",self.class,self.value))
    }
}

type BasicMuonEvent = Event<Real, Data>;



type SuperlativeValue = TimeValue<Real>;

impl SuperlativeValue {
    fn from_min(time: Real) -> SuperlativeValue {
        TimeValue {time, value: Real::default() }
    }
    fn from_max(time: Real) -> SuperlativeValue {
        TimeValue {time, value: Real::MAX }
    }
}

type SuperlativeDiff = TimeValue<RealArray<2>>;

impl SuperlativeDiff {
    fn from_min(time: Real) -> SuperlativeDiff {
        TimeValue {time, value: RealArray::new([Real::default(), Real::default()]) }
    }
    fn from_max(time: Real) -> SuperlativeDiff {
        TimeValue {time, value: RealArray::new([Real::default(), Real::MAX]) }
    }
}

#[derive(Clone,Debug,PartialEq)]
enum Mode {
    Rise,
    Fall,
}

#[derive(Clone,Debug)]
struct State(Mode,SuperlativeValue,SuperlativeDiff);

impl State {
    fn from_mode(mode : Option<Mode>, time: Real, value: Real) -> Option<Self> {
        mode.map(|mode| match mode {
            Mode::Rise => State(Mode::Rise, SuperlativeValue { time, value }, SuperlativeDiff { time, value: RealArray::new([value, Real::default()])}),
            Mode::Fall => State(Mode::Fall, SuperlativeValue { time, value }, SuperlativeDiff { time, value: RealArray::new([value, Real::default()])}),
        })
    }
}

#[derive(Default,Clone)]
struct PotentialMode {
    active: bool,
    mode: Option<Mode>,
    duration : Real,
    min_duration : Real,
}

impl PotentialMode {
    fn set_to(&mut self, mode : Option<Mode>, min_duration: Real) {
        self.active = true;
        if self.mode == mode {
            self.duration += 1.0;
        } else {
            self.mode = mode;
            self.duration = 0.0;
            self.min_duration = min_duration;
        }
    }

    fn is_real(&self) -> bool {
        self.active && self.duration == self.min_duration
    }

    fn reset(&mut self) {
        self.active = false;
    }
}


#[derive(Default,Clone)]
pub struct BasicMuonDetector {
    // Value of the derivative at which an event is said to have been detected
    onset_threshold : Real,
    // Time for which the voltage should rise for the rise to be considered genuine.
    onset_min_duration : Real,
    // Value of the derivative at which an event is said to have peaked
    fall_threshold : Real,
    // Time for which the voltage should drop for the peak to be considered genuine
    fall_min_duration : Real,
    // Value of the derivative at which an event is said to have finished
    termination_threshold : Real,
    // Time for which the voltage should level for the finish to be considered genuine
    termination_min_duration : Real,

    // If a change in signal behavior is detected then it is recorded in potential_mode.
    //If the change lasts the requisite duration then the mode is changed.
    potential_mode : PotentialMode,
    state: Option<State>,
}

impl BasicMuonDetector {
    pub fn new(
        onset_threshold : Real, onset_min_duration : Real,
        fall_threshold : Real, fall_min_duration : Real,
        termination_threshold : Real, termination_min_duration : Real,
    ) -> Self { Self {
        onset_threshold, onset_min_duration,
        fall_threshold, fall_min_duration,
        termination_threshold, termination_min_duration,
        ..Default::default()
    } }

}

impl Detector for BasicMuonDetector {
    type TimeType = Real;
    type ValueType = RealArray<2>;
    type DataType = Data;

    fn signal(&mut self, time: Real, value: RealArray<2>) -> Option<BasicMuonEvent> {
        if let Some(state) = &mut self.state {
            match state {
                State(Mode::Rise, peak, steepest_rise) => {
                    //  Update Steepest Rise
                    if value[1] >= steepest_rise.value[1] {
                        steepest_rise.time = time;
                        steepest_rise.value = value;
                    }
                    //  Update Peak
                    if value[0] >= peak.value {
                        peak.time = time;
                        peak.value = value[0];
                    }
                    if value[1] <= self.fall_threshold {
                        self.potential_mode.set_to(Some(Mode::Fall), self.fall_min_duration);
                    }
                },
                State(Mode::Fall, nadir, sharpest_fall) => {
                    if value[1] <= sharpest_fall.value[1] {
                        sharpest_fall.time = time;
                        sharpest_fall.value = value;
                    }
                    //  Update Nadir
                    if value[0] <= nadir.value {
                        nadir.time = time;
                        nadir.value = value[0];
                    }
    
                    if value[1] >= self.onset_threshold {
                        self.potential_mode.set_to(Some(Mode::Rise), self.onset_min_duration);
                    } else if value[1] >= self.termination_threshold {
                        self.potential_mode.set_to(None, self.termination_min_duration);
                    }
                },
            }
        } else {
            if value[1] >= self.onset_threshold {
                self.potential_mode.set_to(Some(Mode::Rise), self.onset_min_duration);
            }
        }

        if self.potential_mode.is_real() {
            let event = match &self.state {
                Some(State(Mode::Rise, peak, steepest_rise)) => match self.potential_mode.mode {
                    Some(Mode::Rise) => None,
                    Some(Mode::Fall) => Some(Data{ class: Class::Peak, value: peak.value, superlative: Some(steepest_rise.clone()) }.make_event(peak.time)),
                    None             => None
                },
                Some(State(Mode::Fall, nadir, sharpest_fall)) => match self.potential_mode.mode {
                    Some(Mode::Rise) => Some(Data{ class: Class::EndOnset, value: nadir.value, superlative: Some(sharpest_fall.clone()) }.make_event(nadir.time)),
                    Some(Mode::Fall) => None,
                    None => Some(Data{ class: Class::End, value: nadir.value, superlative: Some(sharpest_fall.clone()) }.make_event(nadir.time))
                },
                None => match self.potential_mode.mode {
                    Some(Mode::Rise) => Some(Data{ class: Class::Onset, value: value[0], superlative: None }.make_event(time - self.potential_mode.duration)),
                    Some(Mode::Fall) => None,
                    None => None
                }
            };
            self.state = State::from_mode(self.potential_mode.mode.clone(), time, value[0]);
            self.potential_mode.reset();
            event
        } else {
            None
        }
    }
}

#[derive(Default,Clone, Debug)]
enum AssemblerMode {
    #[default]Waiting,
    Rising { start : TimeValue<Real> },
    Falling { start : TimeValue<Real>, steepest_rise : Option<TimeValue<RealArray<2>>>, peak : TimeValue<Real> },
}

#[derive(Default,Clone)]
pub struct BasicMuonAssembler {
    mode : AssemblerMode,
}

impl Assembler for BasicMuonAssembler {
    type DetectorType = BasicMuonDetector;

    fn assemble_pulses(&mut self, source : Event<Real,Data>) -> Option<Pulse> {
        match self.mode.clone() {
            AssemblerMode::Waiting => {
                match source.get_data().get_class() {
                    Class::Onset => {
                        let start = TimeValue { time: source.get_time(), value: source.get_data().get_value() };
                        self.mode = AssemblerMode::Rising { start };
                        None
                    },
                    _ => None,
                }
            },
            AssemblerMode::Rising { start } => {
                match source.get_data().get_class() {
                    Class::Peak => {
                        let peak = TimeValue::<Real> { time: source.get_time(), value: source.get_data().get_value() };
                        self.mode = AssemblerMode::Falling { start, steepest_rise: source.get_data().get_superlative(), peak };
                        None
                    },
                    _ => None,
                }
            },
            AssemblerMode::Falling { start, steepest_rise, peak } => {
                let end = match source.get_data().get_class() {
                    Class::End => {
                        self.mode = AssemblerMode::Waiting;
                        let end = TimeValue { time: source.get_time(), value: source.get_data().get_value() };
                        Some(end)
                    },
                    Class::EndOnset => {
                        let end = TimeValue { time: source.get_time(), value: source.get_data().get_value() };
                        self.mode = AssemblerMode::Rising { start: end.clone() };
                        Some(end)
                    },
                    _ => None,
                };
                end.map(|end| {
                    let steepest_rise = steepest_rise.unwrap_or_default().into();
                    let sharpest_fall = source.get_data().get_superlative().map(|tv|tv.into()).unwrap_or_default();
                    Pulse {
                        start: start.into(), peak: peak.into(), end: end.into(),
                        steepest_rise, sharpest_fall
                    }
                })
            },
        }
    }
}


#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::processing;
    use super::*;

    #[test]
    fn zero_data() {

        //assert!(results.is_empty());
    }

    #[test]
    fn test_gate_zero_threshold() {
        let data = [4, 3, 2, 5, 6, 1, 5, 7, 2, 4];
    }
}