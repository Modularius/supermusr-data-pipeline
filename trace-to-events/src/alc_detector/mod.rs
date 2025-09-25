mod partition;
mod fitting;

use fitting::{Back2BackExpParams, fit_n_peaks};

use crate::{alc_detector::fitting::Model, pulse_detection::{pulse::TimeValueOptional, Pulse, Real}};
pub(crate) use partition::partition_trace;


#[tracing::instrument(skip_all)]
pub(crate) fn fit_n_peaks_b2bexp<'a>(time: &[Real], intensities: &[Real], num_peaks: usize) -> Vec<Pulse> {
    let sol = fit_n_peaks::<Back2BackExpParams>(time, intensities, num_peaks);
    sol.parameters.iter().map(|params| {
        let params = Back2BackExpParams::new(params.as_slice());
        Pulse {
            start: TimeValueOptional { time: None, value: None },
            end: TimeValueOptional { time: None, value: None },
            peak: TimeValueOptional { time: Some(params.x0), value: Some(params.i) },
            steepest_rise: TimeValueOptional { time: None, value: None },
            sharpest_fall: TimeValueOptional { time: None, value: None },
        }
    }).collect::<Vec<_>>()
}