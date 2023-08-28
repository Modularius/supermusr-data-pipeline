pub mod change_detector;
//pub mod composite;
pub mod pulse_detector;
pub mod threshold_detector;
pub mod peak_detector;

use crate::{events::{event::Event, EventData}, tracedata::{TraceEventData, TraceValue}, trace_iterators::feedback::FeedbackParameter};


pub trait FeedbackDetector : Detector {
    fn modify_parameter(&mut self, _time: Self::TimeType, _param : &FeedbackParameter<Self::ValueType>);
}

pub trait EventValuedDetector : Detector {
    type DataValueType : TraceEventData;

    fn on_event(&mut self, event : Event<Self::DataValueType>) -> Option<Event<Self::DataType>>;
}

pub trait Detector : Clone {
    type TimeType;
    type ValueType : TraceValue;
    type DataType : EventData;
    fn signal(&mut self, time: Self::TimeType, value: Self::ValueType) -> Option<Event<Self::DataType>>;
}
