use crate::Detector;
use crate::{Integer, Real};
use common::Intensity;
use common::Time;
use std::fmt::Debug;
use std::fmt::Display;
use std::slice::Iter;

use self::event::Event;


pub mod event;



pub struct EventIter<I, D> where
    I: Iterator<Item = (D::TimeType, D::ValueType)>,
    D: Detector,
{
    source: I,
    detector: D,
}

impl<I,D> Clone for EventIter<I, D> where
    I: Iterator<Item = (D::TimeType, D::ValueType)> + Clone,
    D: Detector + Clone,
{
    fn clone(&self) -> Self {
        Self { source: self.source.clone(), detector: self.detector.clone() }
    }
}

impl<I, D> Iterator for EventIter<I, D>
where
    I: Iterator<Item = (D::TimeType, D::ValueType)>,
    D: Detector,
{
    type Item = Event<D::DataType>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.source.next() {
            if let Some(event) = self.detector.signal(item.0, item.1) {
                return Some(event);
            }
        }
        None
    }
}


pub trait EventFilter<I, D>
where
    I: Iterator<Item = (D::TimeType, D::ValueType)>,
    D: Detector,
{
    fn events(self, detector: D) -> EventIter<I, D>;
}

impl<I, D> EventFilter<I, D> for I
where
    I: Iterator<Item = (D::TimeType, D::ValueType)>,
    D: Detector,
{
    fn events(self, detector: D) -> EventIter<I, D> {
        EventIter {
            source: self,
            detector,
        }
    }
}










pub trait EventData: Default + Debug + Clone + Display {
    fn has_influence_at(&self, index: Real) -> bool {
        true
    }
    fn get_intensity_at(&self, index: Real) -> Real {
        Real::default()
    }
}