pub mod change_detector;
pub mod composite;
pub mod pulse_detector;
pub mod peak_detector;
//pub mod partitioner;

use crate::{events::{event::Event, EventData}, trace_iterators::feedback::OptFeedParam};

pub trait FeedbackDetector : Detector {
    type ParameterType;
    fn modify_parameter(&mut self, _time: Self::TimeType, _param : OptFeedParam<Self::ParameterType>) {}
}

pub trait Detector {
    type TimeType;
    type ValueType;
    type DataType : EventData;
    fn signal(&mut self, time: Self::TimeType, value: Self::ValueType) -> Option<Event<Self::DataType>>;
}
