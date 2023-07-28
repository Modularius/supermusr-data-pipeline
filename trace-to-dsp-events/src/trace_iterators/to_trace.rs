use crate::{EventIter, Detector, detectors::event::{SingleEvent, EventClass}, peak_detector, Real};


pub trait ToTrace<I,T,V> where I : Iterator {
    fn to_trace(self, length : T) -> Vec<(T,V)>;
}

impl<I> ToTrace<I,Real,Real> for I where I: Iterator<Item = SingleEvent<peak_detector::Class>> {
    fn to_trace(self, length : Real) -> Vec<(Real,Real)> {
        let events : Vec<SingleEvent<peak_detector::Class>> = self.collect();
        let mut it = events.iter();
        (0..length as usize).map(|i| (i as Real, {
            it.next();
            0.
        }
        )).collect()
    }
}