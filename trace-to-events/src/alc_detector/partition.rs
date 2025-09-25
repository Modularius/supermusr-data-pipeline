use std::ops::Range;

use supermusr_common::{Intensity, Time};

use crate::pulse_detection::Real;

pub(crate) type TracePartition = Range<Time>;

pub(crate) fn partition_trace<'a>(mut raw_trace_iter: impl Iterator<Item = (Real, Real)>, peak_estimates: &'a [(Real, Real)]) -> Vec<TracePartition> {
    peak_estimates.windows(2).into_iter().map(|peak| {
        let intensity_of_last_peak = peak.get(0).expect("Window has element, this should never fail.").1;
        let intensity_of_next_peak = peak.get(1).expect("Window has element, this should never fail.").1;
        let lower = raw_trace_iter.position(|(t,x)|x > intensity_of_last_peak).unwrap() as Time;
        let upper = raw_trace_iter.position(|(t,x)|x > intensity_of_next_peak).unwrap() as Time;
        lower..upper
    })
    .collect()
}