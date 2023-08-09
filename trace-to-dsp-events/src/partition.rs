use itertools::Itertools;

use crate::{Real, Detector,
    events::event::Event,
    detectors,
};
use std::{fmt::Display, marker::PhantomData, iter::Peekable};
use std::fmt::Debug;
use std::slice;



//#[derive(Default)]
pub struct TracePartition<I, Det> where
    I : Iterator<Item = (Det::TimeType,Det::ValueType)>,
    Det: Detector,
{
    pub event : Event<Det::DataType>,
    pub iter : I,
    pub length : usize,
}

impl<I, Det> TracePartition<I, Det> where
    I : Iterator<Item = (Det::TimeType,Det::ValueType)> + Clone + Debug,
    Det: Detector,
    Det::TimeType : Default + Debug + PartialEq,
    Det::ValueType : Default + Debug,
    Det::DataType : Clone,
{
    pub fn get_event(&self) -> &Event<Det::DataType> {
        &self.event
    }
    pub fn iter(&self) -> SubPartitionIter<I> {
        SubPartitionIter { iter: self.iter.clone(), length: self.length }
    }
}

impl<I,Det> Clone for TracePartition<I,Det> where
    I : Iterator<Item = (Det::TimeType,Det::ValueType)> + Clone,
    Det: Detector,
    Det::TimeType :  Clone,
    Det::ValueType : Clone,
    Det::DataType : Clone,
{
    fn clone(&self) -> Self {
        Self { event: self.event.clone(), iter: self.iter.clone(), length: self.length.clone() }
    }
}


impl<I,Det> Debug for TracePartition<I,Det> where
    I : Iterator<Item = (Det::TimeType,Det::ValueType)> + Debug,
    Det: Detector,
    Det::TimeType : Debug,
    Det::ValueType : Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TracePartition").field("event", &self.event).field("iter", &self.iter).field("length", &self.length).finish()
    }
}

impl<I, Det> Display for TracePartition<I, Det> where
    I : Iterator<Item = (Det::TimeType,Det::ValueType)> + Clone + Debug,
    Det: Detector,
    Det::TimeType : Default + Clone + Debug,
    Det::ValueType : Default + Clone + Debug,
    Det::DataType : Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("event:{0}, from {1:?} to {2}",self.event, self.iter, self.length))
    }
}







#[derive(Default)]
pub struct SubPartitionIter<I> where I : Iterator {
    iter : I,
    length : usize,
}

impl<I> Iterator for SubPartitionIter<I> where I : Iterator {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.length > 0 {
            self.length -= 1;
            self.iter.next()
        } else {
            None
        }
    }
}







#[derive(Clone)]
pub struct PartitionIter<I, Det> where
    I : Iterator<Item = (Det::TimeType,Det::ValueType)>,
    Det: Detector,
    Det::TimeType : Default + Clone + Debug,
    Det::ValueType : Default + Clone + Debug,
    Det::DataType : Clone,
{
    detector: Det,
    source: I,
}

impl<I, Det> PartitionIter<I, Det> where
    I : Iterator<Item = (Det::TimeType,Det::ValueType)>,
    Det: Detector,
    Det::TimeType : Default + Clone + Debug,
    Det::ValueType : Default + Clone + Debug,
    Det::DataType : Clone,
{
    pub fn new(source: I, detector: Det) -> Self {
        PartitionIter { source, detector }
    }
    #[cfg(test)]
    pub fn get_detector(&self) -> &Det {
        &self.detector
    }
}

impl<I, Det> Iterator for PartitionIter<I,Det> where
    I : Iterator<Item = (Det::TimeType,Det::ValueType)> + Clone + Debug,
    Det: Detector,
    Det::TimeType : Default + Clone + Debug,
    Det::ValueType : Default + Clone + Debug,
    Det::DataType : Clone,
{
    type Item = TracePartition<I,Det>;

    fn next(&mut self) -> Option<Self::Item> {
        let iter = self.source.clone();
        let mut length : usize = 0;
        loop {
            length += 1;
            let val = self.source.next()?;
            match self.detector.signal(val.0,val.1) {
                Some(event) => return Some(TracePartition { event, iter, length }),
                None => (),
            };
        }
    }
}
pub trait PartitionFilter<I,Det> where
    I : Iterator<Item = (Det::TimeType,Det::ValueType)>,
    Det: Detector,
    Det::TimeType : Default + Clone + Debug,
    Det::ValueType : Default + Clone + Debug,
    Det::DataType : Clone,
{
    fn partition_on_detections(self, detector: Det) -> PartitionIter<I,Det>;
}

impl<I, Det> PartitionFilter<I,Det> for I where
    I : Iterator<Item = (Det::TimeType,Det::ValueType)>,
    Det: Detector,
    Det::TimeType : Default + Clone + Debug + 'static,
    Det::ValueType : Default + Clone + Debug + 'static,
    Det::DataType : Clone,
{
    fn partition_on_detections(self, detector: Det) -> PartitionIter<I,Det> {
        PartitionIter::new(self, detector)
    }
}



#[cfg(test)]
mod tests {
    use crate::peak_detector::PeakDetector;

    use super::{PartitionFilter, Real};
    use common::Intensity;
    use super::detectors::change_detector::ChangeDetector;

    #[test]
    fn sample_data() {
        let input: Vec<Intensity> = vec![0, 6, 2, 1, 3, 1, 0];
        let output: Vec<_> = input
            .iter()
            .enumerate()
            .map(|(i, v)| (i as Real, *v as Real))
            .partition_on_detections(PeakDetector::default())
            //.map(|(_, x)| x)
            .collect();
    }
}