use std::fmt::Display;

use crate::change_detector::{ChangeDetector, self};
use crate::events::Event;
use crate::peak_detector::LocalExtremumDetector;
use crate::tracedata::{EventData, Stats};
use crate::{Detector, Real, RealArray, TracePair, peak_detector, ode};
use anyhow::{Result,anyhow};

use super::FeedbackDetector;



#[derive(Default, Debug, Clone)]
pub enum ODESolution {
    #[default]Trivial,
    BiExp{
        amp_1 : Real,
        amp_2 : Real,
        lambda_1 : Real,
        lambda_2 : Real,
        baseline : Real,
    },
    SinCos{
        amp_1 : Real,
        amp_2 : Real,
        lambda : Real,
        theta : Real,
        baseline : Real,
    }
}
impl ODESolution {
    pub fn value(&self, t : Real) -> Real {
        match self {
            ODESolution::BiExp { amp_1, amp_2, lambda_1, lambda_2, baseline } => 
                amp_1*Real::exp(lambda_1*t) + amp_2*Real::exp(lambda_2*t) + baseline,
            ODESolution::SinCos { amp_1, amp_2, lambda, theta, baseline } => 
                Real::exp(lambda*t)*(amp_1*Real::cos(theta*t) + amp_2*Real::sin(theta*t)) + baseline,
            _ => Real::default(),
        }

    }
    fn calc_solution(&mut self, peak : (Real,Real), max_slope : (Real,Real,Real)) {
        match self {
            ODESolution::BiExp { amp_1, amp_2, lambda_1, lambda_2, baseline } => {},
            ODESolution::SinCos { amp_1, amp_2, lambda, theta, baseline } => {
                // 0 = amp_1*(lambda*lambda - theta*theta - 2*lambda*theta*max_slope.0) + amp_2*((lambda*lambda - theta*theta)*max_slope.0 + 2*lambda*theta)
                // max_slope.1*Real::exp(-lambda*max_slope.0) =
                //  amp_1*((lambda*lambda - theta*theta)*Real::cos(theta*max_slope.0) - 2*lambda*theta*Real::sin(theta*max_slope.0))
                // + amp_2*(2*lambda*theta*Real::cos(theta*max_slope.0) + (lambda*lambda - theta*theta)*Real::sin(theta*max_slope.0))
                
                // amp_1 = amp_2*((lambda*lambda - theta*theta)*max_slope.0 + 2*lambda*theta)/(-lambda*lambda + theta*theta + 2*lambda*theta*max_slope.0)

                // max_slope.1*Real::exp(-lambda*max_slope.0)/[
                //    ((lambda*lambda - theta*theta)*Real::cos(theta*max_slope.0) - 2*lambda*theta*Real::sin(theta*max_slope.0))*((lambda*lambda - theta*theta)*max_slope.0 + 2*lambda*theta)/(-lambda*lambda + theta*theta + 2*lambda*theta*max_slope.0)
                // + (2*lambda*theta*Real::cos(theta*max_slope.0) + (lambda*lambda - theta*theta)*Real::sin(theta*max_slope.0))
                // ] =
                //  amp_2
                let d = Real::powi(*lambda,2) - Real::powi(*theta,2);
                let e = 2.0**lambda**theta;
                let cos = Real::cos(*theta*max_slope.0);
                let sin = Real::sin(*theta*max_slope.0);
                *amp_2 = max_slope.1*Real::exp(-*lambda*max_slope.0)/(
                    (d*cos - e*sin)*(d*max_slope.0 + e)/(e*max_slope.0 - d) + (e*cos + d*sin)
                );
                *amp_1 = *amp_2 * (d*max_slope.0 + e)/(d - e*max_slope.0);
                *baseline = peak.1 - Real::exp(*lambda*peak.0)*(*amp_1*Real::cos(*theta*peak.0) + *amp_2*Real::sin(*theta*peak.0));
            },
            _ => {},
        }
    }
}
impl Display for ODESolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ODESolution::BiExp { amp_1, amp_2, lambda_1, lambda_2, baseline } => 
                f.write_fmt(format_args!("1,{amp_1},{amp_2},{lambda_1},{lambda_2},{baseline}")),
            ODESolution::SinCos { amp_1, amp_2, lambda, theta, baseline } => 
                f.write_fmt(format_args!("-1,{amp_1},{amp_2},{lambda},{theta},{baseline}")),
            _ => f.write_fmt(format_args!("")),
        }
    }
}
#[derive(Default, Debug, Clone)]
pub struct ODEData {
    start : Real,
    end : Real,
    quadratic : Real,
    linear : Real,
    constant : Real,
    solution : ODESolution,
    residual : Real,
}
impl ODEData {
    pub fn new(start : Real, end: Real,
        quadratic : Real, linear : Real, constant : Real,
        residual : Real) -> Self {
        ODEData{ start, end,
            quadratic, linear, constant,
            residual, ..Default::default() }
    }
    fn calc_solution(&mut self, peak : (Real,Real), max_slope : (Real,Real,Real)) {
        self.solution.calc_solution(peak,max_slope);
    }
}
impl EventData for ODEData {}

impl Display for ODEData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0},{1},{2},{3},{4},{5},{6}",
            self.start, self.end,
            self.residual
            self.quadratic, self.linear, self.constant,
            self.solution,
        ))
    }
}

pub type ODEEvent = Event<Real, ODEData>;






#[derive(Default,Clone,PartialEq)]
enum Mode {
    #[default]Flat,
    Rising,
    Peaking,
    Falling,
}


#[derive(Default,Clone)]
pub struct MuonDetector {
    mode : Mode,
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
    pub fn init(&mut self, time : Real, baseline : Real, baseslope : Real) {
        println!("Event Initialised at {time}");
        self.baseline = baseline;
        self.max_slope = (0.0, self.baseline, baseslope);
        self.peak = (0.0, self.baseline);
        self.estimator.clear();
        self.start = time;
        self.active = true;
    }
    pub fn new_event(&self, time : Real) -> Option<ODEEvent> {
        println!("Event Finalised at {time}");
        let result = self.estimator.get_parameters();
        //Which order do the coefficients come in?
        match result {
            Ok(ode::Status::Ok((monic_quadratic,residual))) => {
                let (quadratic,linear,constant) = monic_quadratic.get_coefficients();
                let (root1,_) = monic_quadratic.calc_complex_solutions();
                //let amplitude = (self.peak.1 - self.baseline)/biexp_value(biexp_peak_time(kappa, rho), kappa, rho);
                let mut data = ODEData {
                    start: self.start, end: time,
                    quadratic,linear,constant,
                    solution: ODESolution::SinCos { lambda: root1.0, theta: root1.1, 
                        amp_1 : 0.0, amp_2 : 0.0, baseline : 0.0
                    },
                    ..Default::default() };
                data.calc_solution(self.peak, self.max_slope);
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
    type DataType = ODEData;
    
    fn signal(&mut self, time: Self::TimeType, diff: Self::ValueType) -> Option<ODEEvent> {
        match self.mode {
            Mode::Flat => {
                if diff[1] > self.slope_threshold {
                    self.mode = Mode::Rising
                } else if diff[1] < -self.slope_threshold {
                    self.mode = Mode::Falling
                }
            },
            Mode::Rising => if diff[1] < -0.0 { self.mode = Mode::Peaking },
            Mode::Peaking => {
                if diff[1] > self.slope_threshold {
                    self.mode = Mode::Rising
                } else if diff[1] < -self.slope_threshold {
                    self.mode = Mode::Falling
                }
            },
            Mode::Falling => if diff[1] > 0.0 { self.mode = Mode::Flat },
        }
        if self.active {
            if self.mode == Mode::Flat {
                self.active = false;
                self.new_event(time)
            } else {
                if diff[0] > self.peak.1 {
                    self.peak = (time - self.start,diff[0]);
                }
                if diff[1] > self.max_slope.2 {
                    self.max_slope = (time - self.start,diff[0],diff[1]);
                }
                self.estimator.push(
                    diff[0],
                    diff[1],
                    diff[2]
                );
                None
            }
        } else {
            if self.mode == Mode::Rising {
                //let event = self.change_finder.signal(time, diff[0].into())?;
                //use change_detector::ChangeClass as CC;
                //if event.get_data().get_class() == CC::Rising {
                self.init(time, diff[0], diff[1]);
            }
            None
        }
    }
}