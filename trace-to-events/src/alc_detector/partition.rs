use supermusr_common::{Intensity, Time};

use crate::pulse_detection::Real;

pub(crate) struct TracePartition<'a> {
    trace: &'a [(Real, Real)]
}

fn partition_trace<'a>(raw_trace: &'a [Intensity], peak_estimates: &'a [(Time,Intensity)]) -> Vec<&'a [Real]> {
    let mut iter = raw_trace.iter();
    peak_estimates.windows(2).into_iter().map(|peak| {
        let lower = iter.position(|x|*x > peak.get(0).unwrap().1).unwrap();
        let upper = iter.position(|x|*x > peak.get(1).unwrap().1).unwrap();
        raw_trace.iter()
            .enumerate()
            .take(upper)
            .skip(lower)
            .min_by_key(|x|x.1)
            .unwrap()
            .0
    })

    .collect()
}