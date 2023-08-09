// Code from https://github.com/swizard0/smoothed_z_score/blob/master/README.md

/*
iterators of raw trace data have the trait EventFilter<I,S,D> implemented
The events method consumes a raw trace iterator and emits an EventIter iterator
A detector is a struct that

I is an iterator to the enumerated raw trace data, S is the detector signal type and D is the detector.

*/

pub mod detectors;
pub mod event_iterators;
pub mod events;
pub mod tagged;
pub mod partition;
pub mod modifiers;

use std::{collections::VecDeque, iter::Peekable};

use common::Intensity;

pub use detectors::{
    peak_detector,
    Detector
};
pub use events::{
    EventFilter,
    EventIter,
};

pub mod trace_iterators;
pub use trace_iterators::RealArray;

pub mod window;
pub use window::smoothing_window::SmoothingWindow;

pub type Real = f64;
pub type Integer = i16;

pub mod processing {
    use super::*;
    pub fn make_enumerate_real((i, v): (usize, &Intensity)) -> (Real, Real) {
        (i as Real, *v as Real)
    }
    pub fn make_enumerate_integeral((i, v): (Real, Real)) -> (usize, Integer) {
        (i as usize, v as Integer)
    }
}

pub type EnumeratedValue = (Real,Real);

#[cfg(test)]
mod tests {
    use crate::window::composite::CompositeWindow;
    use crate::window::{WindowFilter,trivial::Realisable};
    use common::Intensity;

    use super::trace_iterators::finite_difference::FiniteDifferencesFilter;

    use super::{EventFilter, Real};

    #[test]
    fn sample_data() {
        let input = vec![
            1.0, 1.0, 1.1, 1.0, 0.9, 1.0, 1.0, 1.1, 1.0, 0.9, 1.0, 1.1, 1.0, 1.0, 0.9, 1.0, 1.0,
            1.1, 1.0, 1.0, 1.0, 1.0, 1.1, 0.9, 1.0, 1.1, 1.0, 1.0, 0.9, 1.0, 1.1, 1.0, 1.0, 1.1,
            1.0, 0.8, 0.9, 1.0, 1.2, 0.9, 1.0, 1.0, 1.1, 1.2, 1.0, 1.5, 1.0, 3.0, 2.0, 5.0, 3.0,
            2.0, 1.0, 1.0, 1.0, 0.9, 1.0, 1.0, 3.0, 2.6, 4.0, 3.0, 3.2, 2.0, 1.0, 1.0, 0.8, 4.0,
            4.0, 2.0, 2.5, 1.0, 1.0, 1.0,
        ];
        let output: Vec<_> = input
            .iter()
            .map(|x| (x * 1000.) as Intensity)
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .finite_differences()
            .window(CompositeWindow::<1,Real>::trivial())
            //.events(EventsDetector::new())
            .collect();
        for line in output {
            println!("{line:?}")
        }
    }
}
