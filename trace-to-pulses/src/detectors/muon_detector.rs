use std::fmt::Display;

use crate::change_detector::{ChangeDetector, self};
use crate::events::Event;
use crate::peak_detector::LocalExtremumDetector;
use crate::tracedata::{EventData, Stats};
use crate::{Detector, Real, RealArray, TracePair, peak_detector, ode};
use anyhow::{Result,anyhow};

use super::FeedbackDetector;




fn biexp_value(t : Real, kappa : Real, rho : Real) -> Real {
    Real::exp(-t/kappa) - Real::exp(-t/rho)
}
fn biexp_peak_time(kappa : Real, rho : Real) -> Real {
    (Real::ln(kappa) - Real::ln(rho))/(1./rho - 1./kappa)
}

#[derive(Default, Debug, Clone)]
pub enum BiexpStatus {
    #[default]Good,
    NegativeDiscriminant,
    NegativeRoot,
}

impl Display for BiexpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            BiexpStatus::Good => write!(f,"0"),
            BiexpStatus::NegativeDiscriminant => write!(f,"d"),
            BiexpStatus::NegativeRoot => write!(f,"r"),
        }
    }
}


#[derive(Default, Debug, Clone)]
pub struct BiexpData {
    status: BiexpStatus,
    start : Real,
    end : Real,
    baseline : Real,
    amplitude : Real,
    kappa : Real,
    rho : Real,
    peak : (Real,Real),
    residual : Real,
}
impl BiexpData {
    pub fn new(status: BiexpStatus, start : Real, end: Real, baseline : Real, amplitude : Real, kappa : Real, rho : Real, peak : (Real,Real), residual : Real) -> Self {
        BiexpData{ status, start, end, baseline, amplitude, kappa, rho, peak, residual }
    }
}
impl EventData for BiexpData {}

impl Display for BiexpData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0},{1},{2},{3},{4},{5},{6},{7},{8},{9},{10}",
            self.status, self.start, self.end, self.baseline, self.amplitude, self.kappa, self.rho,
            biexp_peak_time(self.kappa, self.rho), self.peak.0 - self.start,self.peak.1, self.residual
        ))
    }
}

pub type BiexpEvent = Event<Real, BiexpData>;






#[derive(Default,Clone)]
enum Mode {
    #[default]Flat,
    Rising,
    Falling,
}


#[derive(Default,Clone)]
pub struct MuonDetector {
    active : bool,
    start: Real,
    baseline : Real,
    peak: (Real,Real),
    max_slope: (Real,Real,Real),
    min_slope: (Real,Real,Real),
    estimator: ode::ParameterEstimator,
    peak_finder: LocalExtremumDetector,
    change_finder: ChangeDetector,

    threshold: Real,
    slope_threshold: Real,
}

impl MuonDetector {
    pub fn new(threshold : Real, slope_threshold : Real) -> Self {
        Self { active: false, threshold, slope_threshold, change_finder: ChangeDetector::new(slope_threshold), ..Default::default() }
    }
    pub fn init(&mut self, time : Real, baseline : Real) {
        println!("Event Initialised at {time}");
        self.baseline = baseline;
        self.peak = (time, self.baseline);
        self.estimator.clear();
        self.start = time;
        self.active = true;
    }
    pub fn new_event(&self, time : Real) -> Option<BiexpEvent> {
        println!("Event Finalised at {time}");
        let result = self.estimator.get_parameters();
        match result {
            Ok(ode::Status::Ok(((kappa,rho),_,residual))) => {
                let amplitude = (self.peak.1 - self.baseline)/biexp_value(biexp_peak_time(kappa, rho), kappa, rho);
                let data = BiexpData { status: BiexpStatus::Good, start: self.start, end: time, baseline: self.baseline, amplitude, kappa, rho, peak: self.peak, residual };
                Some(data.make_event(self.start))
            },
            Ok(ode::Status::TooShort) => {
                None
            },
            Ok(ode::Status::DiscriminantNonPositive(residual)) => {
                let data = BiexpData { status: BiexpStatus::NegativeDiscriminant, start: self.start, end: time, residual, ..Default::default() };
                Some(data.make_event(self.start))
            },
            Ok(ode::Status::ParameterNonPositive(residual)) => {
                let data = BiexpData { status: BiexpStatus::NegativeRoot, start: self.start, end: time, residual, ..Default::default() };
                Some(data.make_event(self.start))
            },
            _ => None,
        }
        /*if amplitude < self.threshold {
            return None;
        }*/

    }
}

impl Detector for MuonDetector {
    type TimeType = Real;
    type ValueType = RealArray<3>;
    type DataType = BiexpData;
    
    fn signal(&mut self, time: Self::TimeType, diff: Self::ValueType) -> Option<BiexpEvent> {
        if self.active {
            if diff[0] <= self.threshold + self.baseline && diff[1] < -self.slope_threshold {
                self.active = false;
                self.new_event(time)
            } else {
                if diff[0] > self.peak.1 {
                    self.peak = (time,diff[0]);
                }
                self.estimator.push(
                    diff[0],
                    diff[1],
                    diff[2]
                );
                None
            }
        } else {
            if diff[1] > self.slope_threshold {
                //let event = self.change_finder.signal(time, diff[0].into())?;
                //use change_detector::ChangeClass as CC;
                //if event.get_data().get_class() == CC::Rising {
                self.init(time, diff[0]);
            }
            None
        }
    }
}