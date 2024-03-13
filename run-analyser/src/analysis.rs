use anyhow::Result;
use std::fmt::{Display, Formatter};
use supermusr_common::Time;

use crate::message::{EventList, Pair};
/*
pub(crate) struct ValueSd {
    value: f64,
    sd: f64,
}

impl ValueSd {
    fn new(iter: impl Iterator<Item = f64> + Clone, num: f64) -> Self {
        let mean_time = iter.clone().sum::<f64>() / num;
        let sd_time =
            f64::sqrt(iter.map(|t| f64::powi(t - mean_time, 2)).sum::<f64>() / (num - 1.0));
        Self {
            value: mean_time,
            sd: sd_time,
        }
    }
}

impl Display for ValueSd {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{0}~{1}", self.value, self.sd)
    }
} */

use linreg;

pub(crate) struct ChannelAnalysis {
    pub(crate) lifetime: f64,
    pub(crate) num: f64,
    pub(crate) a: f64,
    pub(crate) b: f64,
    pub(crate) bin_error: Vec<f64>,
}

fn calc_lifetime(times: &[Time]) -> f64 {
    times.iter().map(|t| *t as f64).sum::<f64>() / (times.len() - 2) as f64
}

impl<'a> ChannelAnalysis {
    pub(crate) fn new(source : &EventList) -> Self {
        let num = source.time.len() as f64;
        let lifetime = calc_lifetime(source.time.as_slice());
        let max_time = *source.time.iter().max().unwrap_or(&0) as f64;

        let num_bins = 7.0;//f64::floor(num/200.0);
        let bin_size = max_time/num_bins;
        let log_bins = {
            let mut bins = vec![0.0; num_bins as usize + 1];
            for t in &source.time {
                bins[f64::round(*t as f64/bin_size) as usize] += 1.0;
            }
            bins.iter_mut().for_each(|v| if *v != 0.0 { *v = f64::ln(*v); } );
            bins
        };
        let (a,b) = linreg::linear_regression::<_,_,f64>(&(0..log_bins.len()).map(|v|v as f64*bin_size).collect::<Vec<_>>(), &log_bins).unwrap();

        let bin_error = log_bins.iter().enumerate().map(|(i,v)| {
            let time = (i as f64 + 0.5)*bin_size;
            f64::floor(f64::abs(f64::exp(a*time + b) - *v as f64))
        })
        .collect::<Vec<_>>();

        Self { num, lifetime, a, b, bin_error }
    }
}

impl Display for ChannelAnalysis {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "tau = {0}, n = {1}, tau = {2}, N_0 = {3}, \nerror = {4:?}", self.lifetime, self.num, f64::exp(-self.a) - 1.0, f64::exp(self.b), self.bin_error)
    }
}

pub(crate) struct ChannelPairAnalysis {
    pub(crate) detected: ChannelAnalysis,
    pub(crate) simulated: ChannelAnalysis,
}

impl ChannelPairAnalysis {
    pub(crate) fn analyse_channel(pair: &Pair<EventList>) -> Self {
        Self {
            detected: ChannelAnalysis::new(&pair.detected),
            simulated: ChannelAnalysis::new(&pair.simulated),
        }
    }
}

impl Display for ChannelPairAnalysis {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Detected: {0}\nSimulated: {1}", self.detected, self.simulated)
    }
}

pub(crate) struct FramePairAnalysis {
    pub(crate) channels: Vec<ChannelPairAnalysis>,
}

impl Display for FramePairAnalysis {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let channels = self
            .channels
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(";");
        write!(f,"{0}",channels)
    }
}

