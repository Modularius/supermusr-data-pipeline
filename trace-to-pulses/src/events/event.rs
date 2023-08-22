use crate::Real;
use crate::trace_iterators::feedback::FeedbackParameter;
use crate::trace_iterators::feedback::OptFeedParam;
use crate::tracedata::Empty;
use crate::tracedata::TraceEventData;
use crate::tracedata::TraceValue;
use crate::tracedata::TraceData;
use std::fmt::Debug;
use std::fmt::Display;

pub trait EventData: TraceEventData {
    fn make_event(self, time : Real) -> Event<Self> {
        Event::<Self> { time, data: self }
    }
}

impl<D> EventData for D where D : TraceEventData {}

#[derive(Default, Debug, Clone)]
pub struct Event<D> where
    D: EventData,
{
    pub time: Real,
    pub data: D,
}
impl<D> Event<D> where D: EventData,
{
    pub fn new(time: Real, data: D) -> Self { Self { time, data } }
    pub fn get_time(&self) -> Real { self.time }
    pub fn get_data(&self) -> &D { &self.data }
    pub fn get_data_mut(&mut self) -> &mut D { &mut self.data }
    pub fn take_data(self) -> D { self.data }
}

impl<D> PartialEq for Event<D> where D: EventData {
    fn eq(&self, other: &Self) -> bool { self.time == other.time }
}

impl<D> Display for Event<D> where
    D: EventData,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_fmt(format_args!("{0},{1};", self.time, self.data)) }
}

impl<D> TraceData for Event<D> where
    D: EventData,
{
    type TimeType = Real;
    type ValueType = D;
    type DataType = D;

    fn get_time(&self) -> Self::TimeType { self.get_time() }
    fn get_value(&self) -> &Self::ValueType { self.get_data() }
    fn take_value(self) -> Self::ValueType { self.take_data() }
    fn get_data(&self) -> Option<&Self::DataType> { Some(self.get_value()) }
}

















#[derive(Default, Debug, Clone)]
pub struct EventWithFeedback<D,V> where
    D: EventData,
    V : TraceValue,
{
    pub event: Event<D>,
    pub parameter: OptFeedParam<V>,
}
impl<D,V> EventWithFeedback<D,V> where
    D: EventData,
    V : TraceValue,
{
    pub fn new(time: Real, data: D) -> Self {
        Self { event: Event::<D>::new(time, data),..Default::default() }
    }
    pub fn get_time(&self) -> Real { self.event.get_time() }
    pub fn get_data(&self) -> &D { self.event.get_data() }
    pub fn get_data_mut(&mut self) -> &mut D { self.event.get_data_mut() }
    pub fn take_data(self) -> D { self.event.take_data() }
}

impl<D,V> PartialEq for EventWithFeedback<D,V> where
    D: EventData,
    V : TraceValue,
{
    fn eq(&self, other: &Self) -> bool { self.event == other.event }
}

impl<D,V> Display for EventWithFeedback<D,V> where
    D: EventData,
    V : TraceValue,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { std::fmt::Display::fmt(&self.event, f) }
}

impl<D,V> TraceData for EventWithFeedback<D,V> where
    D: EventData,
    V : TraceValue,
{
    type TimeType = Real;
    type ValueType = D;
    type DataType = Empty;

    fn get_time(&self) -> Self::TimeType { self.get_time() }
    fn get_value(&self) -> &Self::ValueType { self.get_data() }
    fn take_value(self) -> Self::ValueType { self.take_data() }
}