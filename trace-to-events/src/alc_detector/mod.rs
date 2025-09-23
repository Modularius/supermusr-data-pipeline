mod partition;
mod fitting;

use fitting::{Back2BackParams, fit_n_peaks, Model};

use crate::pulse_detection::{pulse::TimeValueOptional, Pulse, Real};


pub(crate) fn fit_n_peaks_b2bexp<'a>(time: &[Real], intensities: &[Real], num_peaks: usize) -> Vec<Pulse> {
    let sol = fit_n_peaks::<5,Back2BackParams>(time, intensities, num_peaks);
    sol.parameters.iter().map(|params| {
        let params = Back2BackParams::new(params.as_slice());
        Pulse {
            start: TimeValueOptional { time: None, value: None },
            end: TimeValueOptional { time: None, value: None },
            peak: TimeValueOptional { time: Some(params.x0), value: Some(params.i) },
            steepest_rise: TimeValueOptional { time: None, value: None },
            sharpest_fall: TimeValueOptional { time: None, value: None },
        }
    }).collect::<Vec<_>>()
}