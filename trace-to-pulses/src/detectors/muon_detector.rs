use std::fmt::Display;

use crate::events::Event;
use crate::tracedata::{EventData, Stats};
use crate::{Detector, Real, RealArray};


#[derive(Default, Debug, Clone)]
pub struct MuonData {
    peak : (Real,Real),
    max_slope : (Real,Real,Real),
    min_slope : (Real,Real,Real),
    duration : Real,
}
impl MuonData {
    pub fn new(peak : (Real,Real), max_slope : (Real,Real,Real), min_slope : (Real,Real,Real), duration : Real) -> Self {
        MuonData{ peak, max_slope, min_slope, duration }
    }
}
impl EventData for MuonData {}

impl Display for MuonData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0},{1},{2},{3},{4},{5},{6},{7},{8}",
            self.max_slope.0,self.max_slope.1,self.max_slope.2,
            self.peak.0,self.peak.1,
            self.min_slope.0,self.min_slope.1,self.min_slope.2,
            self.duration
        ))
    }
}

pub type MuonEvent = Event<Real, MuonData>;




#[derive(Default,Clone)]
enum Mode {
    #[default]Flat,
    Rising,
    Falling,
}


#[derive(Default,Clone)]
pub struct MuonDetector {
    mode : Mode,
    start: Real,
    peak: (Real,Real),
    max_slope: (Real,Real,Real),
    min_slope: (Real,Real,Real),

    threshold: Real,
    slope_threshold: Real,
}

impl MuonDetector {
    pub fn new(threshold : Real, slope_threshold : Real) -> Self { Self { threshold,slope_threshold,..Default::default() } }
}

impl Detector for MuonDetector {
    type TimeType = Real;
    type ValueType = RealArray<2>;
    type DataType = MuonData;

    fn signal(&mut self, time: Self::TimeType, value: Self::ValueType) -> Option<MuonEvent> {
        match self.mode {
            Mode::Flat => {
                if value.0[0] > self.threshold {
                    self.start = time;
                    self.max_slope = (time,value.0[0],value.0[1]);
                    self.mode = Mode::Rising;
                }
                None
            },
            Mode::Rising => {
                if self.max_slope.1 < value.0[1] {
                    self.max_slope = (time,value.0[0],value.0[1]);
                }
                if value.0[1] < -self.slope_threshold {
                    self.peak = (time,value.0[0]);
                    self.min_slope = (time,value.0[0], value.0[1]);
                    self.mode = Mode::Falling;
                }
                None
            },
            Mode::Falling => {
                if self.min_slope.1 > value.0[1] {
                    self.min_slope = (time,value.0[0],value.0[1]);
                }
                if value.0[0] < self.threshold {
                    let event = Some(MuonData::new(self.peak, self.max_slope, self.min_slope,time - self.start).make_event(self.start));
                    self.mode = Mode::Flat;
                    event
                } else if value.0[1] > self.slope_threshold {
                    let event = Some(MuonData::new(self.peak, self.max_slope, self.min_slope, time - self.start).make_event(self.start));
                    self.start = time;
                    self.max_slope = (time,value.0[0],value.0[1]);
                    self.mode = Mode::Rising;
                    event
                } else {
                    None
                }
            },
        }
    }
}

