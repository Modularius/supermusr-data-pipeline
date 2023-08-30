use std::marker::PhantomData;

use crate::detectors::{
    Detector,
    FeedbackDetector, EventValuedDetector
};
use crate::trace_iterators::feedback::FeedbackParameter;
use crate::trace_iterators::iter::{TraceIter, TraceIterType};
use crate::tracedata::{TraceData, TraceValue};

use crate::tracedata::EventData;
use super::event::Event;

pub trait EventIterType : Default + Clone {}
#[derive(Default, Clone)]
pub struct Standard;
impl EventIterType for Standard {}
#[derive(Default, Clone)]
pub struct WithFeedback<V>(FeedbackParameter<V>) where V : TraceValue;
impl<V> EventIterType for WithFeedback<V> where V : TraceValue {}
#[derive(Default, Clone)]
pub struct WithTrace;
impl EventIterType for WithTrace {}
#[derive(Default, Clone)]
pub struct WithTraceAndFeedback<V>(FeedbackParameter<V>) where V : TraceValue;
impl<V> EventIterType for WithTraceAndFeedback<V> where V : TraceValue {}

pub struct EventIter<Type, I, D> where
    Type : EventIterType,
    I: Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
{
    source: I,
    detector: D,
    child: Type
}

impl<Type,I,D> Clone for EventIter<Type, I, D> where
    Type : EventIterType,
    I: Iterator + Clone,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
{
    fn clone(&self) -> Self {
        Self { source: self.source.clone(), detector: self.detector.clone(), child: self.child.clone() }
    }
}

impl<I, D> Iterator for EventIter<Standard, I, D> where
    I: Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
{
    type Item = Event<D::TimeType, D::DataType>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let trace = self.source.next()?;
            if let Some(event) = self.detector.signal(trace.get_time(), trace.clone_value()) {
                return Some(event);
            }
        }
    }
}


impl<I, D> Iterator for EventIter<WithFeedback<<I::Item as TraceData>::ValueType>, I, D> where
    I: Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: FeedbackDetector,
{
    type Item = Event<D::TimeType, D::DataType>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let trace = self.source.next()?;
            self.detector.modify_parameter(trace.get_time(), &self.child.0);
            if let Some(event) = self.detector.signal(trace.get_time(), trace.clone_value()) {
                return Some(event);
            }
        }
    }
}


impl<I, D> Iterator for EventIter<WithTrace,I, D> where
    I: Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType> + std::fmt::Debug,
    D: Detector,
{
    type Item = (D::TimeType, D::ValueType, Option<D::DataType>);
    fn next(&mut self) -> Option<Self::Item> {
        let trace = self.source.next()?;
        let event = self.detector.signal(trace.get_time(), trace.clone_value());
        Some((trace.get_time(), trace.take_value(), event.map(|e|e.take_data())))
    }
}

impl<I, D> Iterator for EventIter<WithTraceAndFeedback<<I::Item as TraceData>::ValueType>, I, D> where
    I: Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType, DataType = D::DataValueType> + std::fmt::Debug,
    D: EventValuedDetector + FeedbackDetector,
{
    type Item = Event<D::TimeType, D::DataType>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let trace = self.source.next()?;
            let time = trace.get_time();
            let value = trace.clone_value();
            self.detector.signal(time, value);
            self.detector.modify_parameter(time, &self.child.0);
            if let Some(data) = trace.take_data() {
                if let Some(event) = self.detector.on_event(data.make_event(time)) {
                    return Some(event)
                }
            }
        }
        //Some((trace.get_time(), trace.take_value(), event.map(|e|e.take_data())))
    }
}





pub trait EventFilter<I, D> where
    I: Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
{
    fn events(self, detector: D) -> EventIter<Standard, I, D>;
    fn trace_with_events(self, detector: D) -> EventIter<WithTrace, I, D>;
}

impl<I, D> EventFilter<I, D> for I where
    I: Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
{
    fn events(self, detector: D) -> EventIter<Standard, I, D> {
        EventIter {
            source: self,
            detector,
            child: Standard
        }
    }

    fn trace_with_events(self, detector: D) -> EventIter<WithTrace, I, D> {
        EventIter {
            source: self,
            detector,
            child: WithTrace
        }
    }
}





pub trait EventsWithFeedbackFilter<I, D> where
    I: Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
{
    fn events_with_feedback(self, parameter: FeedbackParameter<<I::Item as TraceData>::ValueType>, detector: D) -> EventIter<WithFeedback<<I::Item as TraceData>::ValueType>, I, D>;
    fn events_from_events_with_feedback(self, parameter: FeedbackParameter<<I::Item as TraceData>::ValueType>, detector: D) -> EventIter<WithTraceAndFeedback<<I::Item as TraceData>::ValueType>, I, D>;
}

impl<I, D> EventsWithFeedbackFilter<I, D> for I where
    I: Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: FeedbackDetector,
{
    fn events_with_feedback(self, parameter: FeedbackParameter<<I::Item as TraceData>::ValueType>, detector: D) -> EventIter<WithFeedback<<I::Item as TraceData>::ValueType>, I, D> {
        EventIter {
            source: self,
            detector,
            child: WithFeedback(parameter)
        }
    }

    fn events_from_events_with_feedback(self, parameter: FeedbackParameter<<I::Item as TraceData>::ValueType>, detector: D) -> EventIter<WithTraceAndFeedback<<I::Item as TraceData>::ValueType>, I, D> {
        EventIter {
            source: self,
            detector,
            child: WithTraceAndFeedback(parameter)
        }
    }
}

