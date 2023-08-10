use crate::detectors::{
    Detector,
    FeedbackDetector
};
use crate::trace_iterators::TraceData;

use super::event::Event;



pub struct EventIter<I, D> where
    I: Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
{
    source: I,
    detector: D,
}

impl<I,D> Clone for EventIter<I, D> where
    I: Iterator + Clone,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector + Clone,
{
    fn clone(&self) -> Self {
        Self { source: self.source.clone(), detector: self.detector.clone() }
    }
}

impl<I, D> Iterator for EventIter<I, D> where
    I: Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
{
    type Item = Event<D::DataType>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let trace = self.source.next()?;
            if let Some(event) = self.detector.signal(trace.get_time(), trace.clone_value()) {
                return Some(event);
            }
        }
    }
}

pub trait EventFilter<I, D> where
    I: Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
{
    fn events(self, detector: D) -> EventIter<I, D>;
}

impl<I, D> EventFilter<I, D> for I where
    I: Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
{
    fn events(self, detector: D) -> EventIter<I, D> {
        EventIter {
            source: self,
            detector,
        }
    }
}















pub struct EventWithFeedbackIter<I, D> where
    I: Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
{
    source: I,
    detector: D,
}

impl<I,D> Clone for EventWithFeedbackIter<I, D> where
    I: Iterator + Clone,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector + Clone,
{
    fn clone(&self) -> Self {
        Self { source: self.source.clone(), detector: self.detector.clone() }
    }
}

impl<I, D> Iterator for EventWithFeedbackIter<I, D> where
    I: Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType, ParameterType = D::ParameterType>,
    D: FeedbackDetector,
{
    type Item = Event<D::DataType>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let trace = self.source.next()?;
            self.detector.modify_parameter(trace.get_time(), trace.get_parameter());
            if let Some(event) = self.detector.signal(trace.get_time(), trace.clone_value()) {
                return Some(event);
            }
        }
    }
}

pub trait EventWithFeedbackFilter<I, D> where
    I: Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType, ParameterType = D::ParameterType>,
    D: FeedbackDetector,
{
    fn events_with_feedback(self, detector: D) -> EventWithFeedbackIter<I, D>;
}

impl<I, D> EventWithFeedbackFilter<I, D> for I where
    I: Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType, ParameterType = D::ParameterType>,
    D: FeedbackDetector,
{
    fn events_with_feedback(self, detector: D) -> EventWithFeedbackIter<I, D> {
        EventWithFeedbackIter {
            source: self,
            detector,
        }
    }
}
