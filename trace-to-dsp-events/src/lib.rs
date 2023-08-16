// Code from https://github.com/swizard0/smoothed_z_score/blob/master/README.md

/*
iterators of raw trace data have the trait EventFilter<I,S,D> implemented
The events method consumes a raw trace iterator and emits an EventIter iterator
A detector is a struct that

I is an iterator to the enumerated raw trace data, S is the detector signal type and D is the detector.

*/

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
    EventWithFeedbackFilter,
    EventWithFeedbackIter,
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
