pub mod change_detector;
//pub mod composite;
pub mod pulse_detector;
pub mod threshold_detector;
pub mod peak_detector;

use crate::{events::{event::Event, EventData}, trace_iterators::feedback::OptFeedParam, tracedata::{TraceEventData, TraceData}};


pub trait FeedbackDetector : Detector {
    fn modify_parameter(&mut self, _time: Self::TimeType, _param : OptFeedParam<<Self::ValueType as TraceData>::ValueType>);
}

pub trait EventValuedDetector : Detector {
    type DataValueType : TraceEventData;

    fn on_event(&mut self, event : Event<Self::DataValueType>) -> Option<Event<Self::DataType>>;
}

pub trait Detector : Clone {
    type TimeType;
    type ValueType;
    type DataType : EventData;
    fn signal(&mut self, time: Self::TimeType, value: Self::ValueType) -> Option<Event<Self::DataType>>;
}
