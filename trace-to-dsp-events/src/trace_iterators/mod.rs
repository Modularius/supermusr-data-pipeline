pub mod finite_difference;
pub mod find_baseline;

pub mod load_from_trace_file;
pub mod save_to_file;
pub mod to_trace;
pub mod feedback;

use crate::Real;
pub type RealArray<const N: usize> = [Real; N];

use std::{fmt::Debug, cell::RefCell, rc::Rc};

use feedback::{
    FeedbackParameter,
    OptFeedParam,
};

pub trait TraceData : Clone {
    type TimeType : Copy + Debug;
    type ValueType : Clone + Debug;
    type ParameterType : Clone + Debug;

    fn get_time(&self) -> Self::TimeType;
    fn get_value(&self) -> &Self::ValueType;
    fn take_value(self) -> Self::ValueType;

    fn clone_value(&self) -> Self::ValueType { self.get_value().clone() }
    fn get_parameter(&self) -> OptFeedParam<Self::ParameterType> { None }
}

impl<X,Y> TraceData for (X,Y) where X : Copy + Debug, Y: Clone + Debug {
    type TimeType = X;
    type ValueType = Y;
    type ParameterType = Y;

    fn get_time(&self) -> Self::TimeType { self.0 }
    fn get_value(&self) -> &Self::ValueType { &self.1 }
    fn take_value(self) -> Self::ValueType { self.1 }
}