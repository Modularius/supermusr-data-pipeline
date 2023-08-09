use crate::Detector;
use crate::EventIter;
use crate::{Integer, Real};
use common::Intensity;
use common::Time;
use std::fmt::Debug;
use std::fmt::Display;
use std::slice::Iter;

use super::EventWithData;
use super::{
    Event,
    EventData,
    event::SimpleEvent
};




#[derive(Default,Clone,Debug)]
pub struct MultipleData<D> where
    D: EventData,
{
    data : Vec<D>,
}

impl<D> MultipleData<D> where
    D: EventData,
{
    pub fn new(data : Vec<D>) -> Self {
        MultipleData::<D>{ data }
    }
    fn iter(&self) -> std::slice::Iter<'_,D> {
        self.data.iter()
    }
}


impl<D> IntoIterator for MultipleData<D> where
    D: EventData,
{
    type Item = D;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}



impl<D> EventData for MultipleData<D> where
    D: EventData,
{}

impl<D> Display for MultipleData<D> where
    D: EventData,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for data in &self.data {
            f.write_fmt(format_args!("{data}"))?;
        }
        Ok(())
    }
}






#[derive(Debug, Clone)]
pub struct MultipleEvents<D> where
    D: EventData,
{
    time: Real,
    data: MultipleData<D>,
}
impl<D> MultipleEvents<D> where
    D: EventData,
{
    pub fn new(data: MultipleData<D>, time: Real) -> Self {
        Self { data, time }
    }
}
impl<D> PartialEq for MultipleEvents<D> where D: EventData, {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}
impl<D> Event for MultipleEvents<D> where
    D: EventData,
{
    fn get_time(&self) -> Real {
        self.time
    }
    fn has_influence_at(&self, time: Real) -> bool {
        self.data.iter().any(|data| SimpleEvent::<D>{ time: self.time, data: data.clone() }.has_influence_at(time))
    }
    fn get_intensity_at(&self, time: Real) -> Real {
        self.data.iter().map(|data| SimpleEvent::<D>{ time: self.time, data: data.clone() }.get_intensity_at(time)).sum()
    }
}

impl<D> EventWithData for MultipleEvents<D> where
    D: EventData,
{
    type DataType = MultipleData<D>;

    fn get_data(&self) -> &Self::DataType {
        &self.data
    }

    fn get_data_mut(&mut self) -> &mut Self::DataType {
        &mut self.data
    }

    fn take_data(self) -> Self::DataType {
        self.data
    }
}
impl<D> Display for MultipleEvents<D> where
    D: EventData,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for datum in self.data.iter() {
            f.write_fmt(format_args!("{datum}"))?;
        }
        Ok(())
    }
}

pub struct MultipleEventsIntoIterator<E>
where
    E: Event,
{
    source: std::vec::IntoIter<E>,
}

impl<E> Iterator for MultipleEventsIntoIterator<E>
where
    E: Event,
{
    type Item = E;
    fn next(&mut self) -> Option<Self::Item> {
        self.source.next()
    }
}

impl<D> IntoIterator for MultipleEvents<D>
where
    D: EventData + 'static,
{
    type Item = SimpleEvent<D>;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.data.into_iter().map(move |d|SimpleEvent::<D>::new(self.time, d)))
    }
}
