use anyhow::Result;
use std::fmt::{Display, Formatter};
use supermusr_common::Channel;
use crate::{base::EventList, message_pair::MessagePairVector};

impl EventList {
    fn analyse(&self) -> (usize, f64) {
        (self.time.len(), self.time.iter().map(|t|*t as f64).sum::<f64>()/(self.time.len() - 2) as f64)
    }
}

pub(crate) struct ValueSd {
    value : f64,
    sd: f64,
}
impl ValueSd {
    fn new(iter : impl Iterator<Item = f64> + Clone, num : f64) -> Self {
        let mean_time = iter.clone().sum::<f64>()/num;
        let sd_time = f64::sqrt(iter.map(|t|f64::powi(t - mean_time,2)).sum::<f64>()/(num - 1.0));
        Self { value: mean_time, sd: sd_time }
    }
}

impl Display for ValueSd {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{0},{1}", self.value, self. sd)
    }
}

pub(crate) struct ChannelAnalysis {
    pub(crate) lifetime : ValueSd,
    pub(crate) num : ValueSd,
}

impl Display for ChannelAnalysis {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{0}, {1}", self.lifetime, self.num)
    }
}

pub(crate) struct ChannelPairAnalysis {
    pub(crate) detected : ChannelAnalysis,
    pub(crate) simulated : ChannelAnalysis,
}

impl Display for ChannelPairAnalysis {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{0},  {1}", self.detected, self.simulated)
    }
}

pub(crate) struct FramePairAnalysis {
    pub(crate) time : ValueSd,
    pub(crate) time_per_byte_in : ValueSd,
    pub(crate) time_per_byte_out : ValueSd,
    pub(crate) list : Vec<ChannelPairAnalysis>
}

impl Display for FramePairAnalysis {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let channels = self.list.iter().map(|c|
            c.to_string()
        ).collect::<Vec<_>>().join(&",   ");
        writeln!(f, "{0},{1},{2},    {3}", self.time, self.time_per_byte_in, self.time_per_byte_out, channels)
    }
}

pub(crate) fn analyse(vec: &MessagePairVector) -> FramePairAnalysis {
    let (time, time_per_byte_in, time_per_byte_out) = analyse_times(vec);
    let channels = vec.first()
        .map(|m|m.channels.keys().collect::<Vec<&Channel>>())
        .unwrap_or_default();
    let list = channels.into_iter()
        .map(|c|analyse_channel(*c, vec))
        .collect();
    
    FramePairAnalysis {
        time,
        time_per_byte_in,
        time_per_byte_out,
        list,
    }
}

pub(crate) fn analyse_channel(channel_id: Channel, vec: &MessagePairVector) -> ChannelPairAnalysis {
    let num = vec.len() as f64;
    let iter = vec.iter().map(|v| {
        let val = v.channels.get(&channel_id).unwrap();
        (val.detected.analyse(),val.simulated.analyse())
    });
    ChannelPairAnalysis {
        detected: ChannelAnalysis {
            lifetime: ValueSd::new(iter.clone().map(|((t,_),_)|t as f64),num),
            num: ValueSd::new(iter.clone().map(|((_,t),_)|t as f64),num),
        },
        simulated: ChannelAnalysis {
            lifetime: ValueSd::new(iter.clone().map(|(_,(t,_))|t as f64),num),
            num: ValueSd::new(iter.map(|(_,(_,t))|t as f64),num),
        }
    }
}

pub(crate) fn analyse_times(vec: &MessagePairVector) -> (ValueSd,ValueSd,ValueSd) {
    let num = vec.len() as f64;
    
    let times : Vec<(f64,f64,f64)> = vec.iter().map(|v|{
        let time = v.headers
            .get("trace-to-events: time_ns")
            .map(|s| s.parse().ok())
            .flatten()
            .unwrap_or_default();
        
        let bytes_in : f64 = v.headers
            .get("trace-to-events: size of trace")
            .map(|s| s.parse().ok())
            .flatten()
            .unwrap_or_default();
        
        let bytes_out : f64 = v.headers
            .get("trace-to-events: size of events list")
            .map(|s| s.parse().ok())
            .flatten()
            .unwrap_or_default();
        (time,time/bytes_in, time/bytes_out)
    }).collect();
    
    let time = ValueSd::new(times.iter().map(|(t,_,_)|*t), num);
    let time_per_byte_in = ValueSd::new(times.iter().map(|(_,t,_)|*t), num);
    let time_per_byte_out = ValueSd::new(times.iter().map(|(_,_,t)|*t), num);
    (time, time_per_byte_in, time_per_byte_out)
}
