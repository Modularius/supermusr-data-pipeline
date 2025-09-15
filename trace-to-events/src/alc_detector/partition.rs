use std::ops::Range;

use supermusr_common::{Intensity, Time};

pub(crate) type TracePartition = Range<Time>;

fn partition_trace<'a>(raw_trace: &'a [Intensity], peak_estimates: &'a [(Time,Intensity)]) -> Vec<TracePartition> {
    let mut iter = raw_trace.iter();
    peak_estimates.windows(2).into_iter().map(|peak| {
        let lower = iter.position(|x|*x > peak.get(0).unwrap().1).unwrap() as Time;
        let upper = iter.position(|x|*x > peak.get(1).unwrap().1).unwrap() as Time;
        lower..upper
    })
    .collect()
}