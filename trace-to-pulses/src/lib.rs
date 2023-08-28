//! This crate provides tools for converting raw trace data into
//! a stream of events which represent pulses in the trace stream.
//! 
//! A raw trace takes the form of a Vec (or some other similar container)
//! of scalar values. Typical usage of this crate may look like:
//! ```rust
//! let events = trace.iter()
//!     .enumerate()
//!     .map(make_real_enumerate)                       // converts to (Real,Real) format
//!     .window(SmoothedWindow(4))                      // A moving average window of length 4
//!     .events(PulseDetector(ChangeDetector(0.5),1))   // Registers an event when the averaged
//!                                                     // signal changes by 0.5*sigma, where sigma is
//!                                                     // the standard deviation from the moving
//!                                                     //average window
//! ```


pub mod detectors;
pub mod events;
pub mod tracedata;
//pub mod partition;

use common::Intensity;

pub use detectors::{
    peak_detector,
    pulse_detector,
    change_detector,
    Detector
};
pub use events::{
    EventFilter,
    EventIter,
    EventsWithTraceFilter,
};

pub mod trace_iterators;

pub mod window;
pub use window::smoothing_window::SmoothingWindow;

pub type Real = f64;
pub type Integer = i16;
pub type RealArray<const N: usize> = [Real; N];

pub mod processing {
    use super::*;
    pub fn make_enumerate_real((i, v): (usize, &Intensity)) -> (Real, Real) {
        (i as Real, *v as Real)
    }
    pub fn make_enumerate_integeral((i, v): (Real, Real)) -> (usize, Integer) {
        (i as usize, v as Integer)
    }
}

pub fn log_then_panic_t<T>(string: String) -> T {
    log::error!("{string}");
    panic!("{string}");
}

#[cfg(test)]
mod tests {
    //use crate::window::composite::CompositeWindow;
    use common::Intensity;

    use super::*;

    #[test]
    fn sample_data() {
        let input = vec![
            1.0, 1.0, 1.1, 1.0, 0.9, 1.0, 1.0, 1.1, 1.0, 0.9, 1.0, 1.1, 1.0, 1.0, 0.9, 1.0, 1.0,
            1.1, 1.0, 1.0, 1.0, 1.0, 1.1, 0.9, 1.0, 1.1, 1.0, 1.0, 0.9, 1.0, 1.1, 1.0, 1.0, 1.1,
            1.0, 0.8, 0.9, 1.0, 1.2, 0.9, 1.0, 1.0, 1.1, 1.2, 1.0, 1.5, 1.0, 3.0, 2.0, 5.0, 3.0,
            2.0, 1.0, 1.0, 1.0, 0.9, 1.0, 1.0, 3.0, 2.6, 4.0, 3.0, 3.2, 2.0, 1.0, 1.0, 0.8, 4.0,
            4.0, 2.0, 2.5, 1.0, 1.0, 1.0,
        ];
        let output = input
            .iter()
            .map(|x| (x * 1000.) as Intensity)
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real));
            //.finite_differences()
            //.window(CompositeWindow::<1,Real>::trivial())
            //.events(EventsDetector::new())
            //.collect();
        for line in output {
            println!("{line:?}")
        }
    }
}
