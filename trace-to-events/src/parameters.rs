use std::str::FromStr;

use clap::{Parser, Subcommand};
use common::Time;
use trace_to_pulses::Real;
use anyhow::{anyhow, Error};


#[derive(Default, Debug, Clone)]
pub struct ThresholdDuration {
    pub threshold : Real,
    pub duration : Time,
}

impl FromStr for ThresholdDuration {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const ERROR : Error  = anyhow!("Incorrect number of parameters, expected pattern *,*");
        let vals : Vec<_> = s.split(",").collect();
        Ok(ThresholdDuration{
            threshold: Real::from_str(*vals.get(0).ok_or(ERROR)?)?,
            duration: Time::from_str(vals.get(1).ok_or(ERROR)?)?
        })
    }
}

#[derive(Default, Debug, Clone, Parser)]
pub struct SimpleParameters {
    pub threshold_trigger: ThresholdDuration,
}



#[derive(Default, Debug, Clone,Parser)]
pub struct BasicParameters {
    pub gate_size: Real,
    pub min_voltage: Real,
    pub smoothing_window_size: usize,
    pub baseline_length: usize,
    pub max_amplitude: Option<Real>,
    pub min_amplitude: Option<Real>,
    pub muon_onset: ThresholdDuration,
    pub muon_fall: ThresholdDuration,
    pub muon_termination: ThresholdDuration,
}


#[derive(Subcommand, Debug)]
pub enum Mode {
    Simple(SimpleParameters),
    Basic(BasicParameters),
}