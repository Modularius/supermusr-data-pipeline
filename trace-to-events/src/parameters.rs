use std::str::FromStr;

use clap::{Parser, Subcommand};
use common::Time;
use trace_to_pulses::{Real, detectors::threshold_detector::ThresholdDuration};
use anyhow::{anyhow, Error};


#[derive(Default, Debug, Clone)]
pub struct ThresholdDurationWrapper(pub(crate)ThresholdDuration);

impl FromStr for ThresholdDurationWrapper {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vals : Vec<_> = s.split(",").collect();
        Ok(ThresholdDurationWrapper(ThresholdDuration{
            threshold: Real::from_str(*vals.get(0).ok_or(anyhow!("Incorrect number of parameters in threshold, expected pattern '*,*', got '{s}'"))?)?,
            duration: Time::from_str(vals.get(1).ok_or(anyhow!("Incorrect number of parameters in duration, expected pattern '*,*', got '{s}'"))?)? as usize
        }))
    }
}

#[derive(Default, Debug, Clone, Parser)]
pub struct SimpleParameters {
    pub threshold_trigger: ThresholdDurationWrapper,
}


pub struct SaveOptions {
    pub file_name : String,
}


#[derive(Default, Debug, Clone,Parser)]
pub struct BasicParameters {
    pub gate_size: Real,
    pub min_voltage: Real,
    pub smoothing_window_size: usize,
    pub baseline_length: usize,
    pub max_amplitude: Option<Real>,
    pub min_amplitude: Option<Real>,
    pub muon_onset: ThresholdDurationWrapper,
    pub muon_fall: ThresholdDurationWrapper,
    pub muon_termination: ThresholdDurationWrapper,
}


#[derive(Subcommand, Debug)]
pub enum Mode {
    Simple(SimpleParameters),
    Basic(BasicParameters),
}