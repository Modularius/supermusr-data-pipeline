use anyhow::Result;
use std::fmt::{Display, Formatter};
use supermusr_common::{Channel, Time};

use crate::message::EventList;

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
}

pub(crate) struct ChannelAnalysis {
    pub(crate) lifetime: f64,
    pub(crate) num: f64,
    pub(crate) bins: Vec<usize>,
    pub(crate) bin_error: Vec<f64>,
}

fn calc_lifetime(times: &[Time]) -> f64 {
    times.iter().map(|t| *t as f64).sum::<f64>() / (times.len() - 2) as f64
}

/*
fn calc_histogram(times : Vec<Time>, intensities : Vec<Intensity>, num_bins : usize) -> Vec<usize> {
    let max_time = *times.iter().max().as_deref().unwrap_or(&0) as f64;
    let mut bins = vec![num_bins; 0];
    for t in times {
        let bin_index = (bins.len() as f64 * t as f64/max_time) as usize;
        bins[bin_index] += 1;
    }
    bins
}
*/

impl<'a> ChannelAnalysis {
    pub(crate) fn new(source : &EventList) -> Self {
        let num = source.time.len() as f64;
        let lifetime = calc_lifetime(source.time.as_slice());
        let max_time = *source.time.iter().max().unwrap() as f64;

        let num_bins = f64::floor(num/50.0);
        let bin_size = max_time/num_bins;
        let bins = {
            let mut bins = vec![0; num_bins as usize + 1];
            for t in source.time {
                bins[f64::round(t as f64/bin_size) as usize] += 1;
            }
            bins
        };
        let bin_error = bins.iter().enumerate().map(|(i,v)| {
            let time = (i as f64 + 0.5)*bin_size;
            let mut bins = vec![0.0; num_bins as usize + 1];
        })
        .collect::<Vec<_>>();

        Self { num, lifetime, bins, bin_error }
    }
}

impl Display for ChannelAnalysis {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{0},{1}", self.lifetime, self.num)
    }
}

pub(crate) struct ChannelPairAnalysis {
    pub(crate) detected: ChannelAnalysis,
    pub(crate) simulated: ChannelAnalysis,
}

impl Display for ChannelPairAnalysis {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{0}:{1}", self.detected, self.simulated)
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
            .map(|c| format!("{c}"))
            .collect::<Vec<_>>()
            .join(";");
        write!(f,"{0}",channels)
    }
}

pub(crate) fn analyse(vec: &MessagePairVector) -> FramePairAnalysis {
    let (time, time_per_byte_in, time_per_byte_out) = analyse_times(vec);
    let channels = vec
        .first()
        .map(|m| m.channels.keys().collect::<Vec<&Channel>>())
        .unwrap_or_default();
    let channels = channels
        .into_iter()
        .map(|c| analyse_channel(*c, vec))
        .collect();

    FramePairAnalysis {
        channels,
    }
}

pub(crate) fn analyse_channel(channel_id: Channel, vec: &MessagePairVector) -> ChannelPairAnalysis {
    let num = vec.len() as f64;
    ChannelPairAnalysis {
        detected: ChannelAnalysis::new(
            vec.iter()
                .map(|v| &v.channels.get(&channel_id).unwrap().detected),
            num,
        ),
        simulated: ChannelAnalysis::new(
            vec.iter()
                .map(|v| &v.channels.get(&channel_id).unwrap().simulated),
            num,
        ),
    }
}

pub(crate) fn analyse_times(vec: &MessagePairVector) -> (ValueSd, ValueSd, ValueSd) {
    let num = vec.len() as f64;

    let times: Vec<(f64, f64, f64)> = vec
        .iter()
        .map(|v| {
            let time = v
                .headers
                .get("trace-to-events: time_ns")
                .and_then(|s| s.parse().ok())
                .unwrap_or_default();

            let bytes_in: f64 = v
                .headers
                .get("trace-to-events: size of trace")
                .and_then(|s| s.parse().ok())
                .unwrap_or_default();

            let bytes_out: f64 = v
                .headers
                .get("trace-to-events: size of events list")
                .and_then(|s| s.parse().ok())
                .unwrap_or_default();
            (time, time / bytes_in, time / bytes_out)
        })
        .collect();

    let time = ValueSd::new(times.iter().map(|(t, _, _)| *t), num);
    let time_per_byte_in = ValueSd::new(times.iter().map(|(_, t, _)| *t), num);
    let time_per_byte_out = ValueSd::new(times.iter().map(|(_, _, t)| *t), num);
    (time, time_per_byte_in, time_per_byte_out)
}
