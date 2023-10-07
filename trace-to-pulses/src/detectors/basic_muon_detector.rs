use std::fmt::Display;

use crate::events::Event;
use crate::pulse::{Pulse, TimeValue, TimeValueOptional};
use crate::tracedata::{EventData, Stats, TraceValue};
use crate::{Detector, Real, RealArray};

use super::Assembler;

#[derive(Default, Debug, Clone,PartialEq)]
pub enum Class {
    #[default] Onset,
    Steepest,
    Peak,
    End,
    EndOnset,
}
impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Onset => "0",
            Self::Steepest => "1",
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
    pub fn get_superlative(&self) -> Option<TimeValue<RealArray<2>>> { self.superlative }
}
impl EventData for Data {}

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0},{1}",self.class,self.value))
    }
}

type BasicMuonEvent = Event<Real, Data>;




#[derive(Default, Clone, PartialEq)]
enum Mode {
    #[default]Level,
    Rise,
    Fall,
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
    potential_mode : Mode,
    potential_duration : Real,
    mode: Mode,

    steepest_rise : TimeValue<RealArray<2>>,
    sharpest_fall : TimeValue<RealArray<2>>,
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

    fn set_potential_mode_to(&mut self, mode : Mode, extra_mode : Option<Mode>) {
        if self.potential_mode == mode {
            self.potential_duration += 1.0;
        } else if extra_mode.map(|m| self.potential_mode == m).unwrap_or(false) {
            self.potential_mode = mode;
        } else {
            self.potential_mode = mode;
            self.potential_duration = 0.0;
        }
    }
    fn realise_potential_mode_if(&mut self, duration : Real, mode : Mode, class : Class, value : Real, superlative : Option<TimeValue<RealArray<2>>>) -> Option<Data> {
        if self.potential_duration >= duration {
            self.mode = mode;
            Some(Data { class, value, superlative })
        } else {
            None
        }
    }
}

impl Detector for BasicMuonDetector {
    type TimeType = Real;
    type ValueType = RealArray<2>;
    type DataType = Data;

    fn signal(&mut self, time: Real, value: RealArray<2>) -> Option<BasicMuonEvent> {
        match self.mode {
            Mode::Level => {
                if value[1] >= self.onset_threshold {
                    self.set_potential_mode_to(Mode::Rise, None);
                }
            }
            Mode::Rise => {
                if value[1] > self.steepest_rise.value[1] {
                    self.steepest_rise = TimeValue { time, value };
                }
                if value[1] <= self.fall_threshold {
                    self.set_potential_mode_to(Mode::Fall, None);
                }
            },
            Mode::Fall => {
                if value[1] < self.sharpest_fall.value[1] {
                    self.sharpest_fall = TimeValue { time, value };
                }

                if value[1] >= self.onset_threshold {
                    self.set_potential_mode_to(Mode::Rise, None);
                } else if value[1] >= self.termination_threshold {
                    self.set_potential_mode_to(Mode::Level, None);
                }
            },
        }
        match self.mode {
            Mode::Level => match self.potential_mode {
                Mode::Level => None,
                Mode::Rise => self.realise_potential_mode_if(self.onset_min_duration, Mode::Rise, Class::Onset, value[0], None),
                Mode::Fall => None,
            },
            Mode::Rise => match self.potential_mode {
                Mode::Level => None,
                Mode::Rise => None,
                Mode::Fall => self.realise_potential_mode_if(self.fall_min_duration, Mode::Fall, Class::Peak, value[0], Some(self.steepest_rise)),
            },
            Mode::Fall => match self.potential_mode {
                Mode::Level => self.realise_potential_mode_if(self.termination_min_duration, Mode::Level, Class::End, value[0], Some(self.sharpest_fall)),
                Mode::Rise=> self.realise_potential_mode_if(self.onset_min_duration, Mode::Rise, Class::EndOnset, value[0], Some(self.sharpest_fall)),
                Mode::Fall => None,
            },
        }.map(|data|data.make_event(time - self.potential_duration))
    }
}

#[derive(Default,Clone, Debug)]
enum AssemblerMode {
    #[default]Waiting,
    Rising { start : TimeValue<Real>, steepest_rise : Option<TimeValue<RealArray<2>>> },
    Falling { start : TimeValue<Real>, steepest_rise : Option<TimeValue<RealArray<2>>>, peak : TimeValue<Real>, sharpest_fall : Option<TimeValue<RealArray<2>>> },
}

#[derive(Default,Clone)]
pub struct BasicMuonAssembler {
    mode : AssemblerMode,
}

impl Assembler for BasicMuonAssembler {
    type DetectorType = BasicMuonDetector;

    fn assemble_pulses(&mut self, source : Event<Real,Data>) -> Option<Pulse> {
        println!("{0:?}",self.mode);
        match self.mode.clone() {
            AssemblerMode::Waiting => {
                match source.get_data().get_class() {
                    Class::Onset => {
                        let start = TimeValue { time: source.get_time(), value: source.get_data().get_value() };
                        self.mode = AssemblerMode::Rising { start, steepest_rise: None };
                        None
                    },
                    _ => None,
                }
            },
            AssemblerMode::Rising { start, steepest_rise } => {
                match source.get_data().get_class() {
                    Class::Steepest => None/*{
                        let steepest_rise = TimeValue::<RealArray<2>> { time: Some(source.get_time()), value: Some(source.get_data().get_value()) };
                        self.mode = AssemblerMode::Rising { start, steepest_rise };
                        None
                    }*/,
                    Class::Peak => {
                        let peak = TimeValue::<Real> { time: source.get_time(), value: source.get_data().get_value() };
                        self.mode = AssemblerMode::Falling { start, steepest_rise, peak, sharpest_fall: None };
                        None
                    },
                    _ => None,
                }
            },
            AssemblerMode::Falling { start, steepest_rise, peak, sharpest_fall } => {
                match source.get_data().get_class() {
                    Class::End => {
                        self.mode = AssemblerMode::Waiting;
                        let end = TimeValue { time: source.get_time(), value: source.get_data().get_value() };
                        Some(Pulse {
                            start: start.into(), peak: peak.into(), end: end.into(),
                            steepest_rise: steepest_rise.unwrap_or_default().into(),
                            sharpest_fall: sharpest_fall.unwrap_or_default().into()
                        })
                    },
                    Class::EndOnset => {
                        let end = TimeValue { time: source.get_time(), value: source.get_data().get_value() };
                        self.mode = AssemblerMode::Rising { start: end.clone(), steepest_rise: None };
                        Some(Pulse {
                            start: start.into(), peak: peak.into(), end: end.into(),
                            steepest_rise: steepest_rise.unwrap_or_default().into(),
                            sharpest_fall: sharpest_fall.unwrap_or_default().into()
                        })
                    },
                    _ => None,
                }
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