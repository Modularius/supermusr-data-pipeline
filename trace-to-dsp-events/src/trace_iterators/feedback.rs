use std::{cell::RefCell, rc::Rc};

use super::TraceData;
use std::fmt::Debug;

type FeedbackValueType<I> = <<I as Iterator>::Item as TraceData>::ValueType;
type FeedbackFunctionType<I> = fn(&FeedbackValueType<I>, &FeedbackValueType<I>)->FeedbackValueType<I>;

#[derive(Debug)]
pub struct FeedbackParameter<V>(Rc<RefCell<V>>);
pub type OptFeedParam<V> = Option<FeedbackParameter<V>>;

impl<V> FeedbackParameter<V> {
    pub fn new(initial : V) -> Self {
        Self(Rc::new(RefCell::new(initial)))
    }
    pub fn set(&self, value : V) {
        *(*self.0).borrow_mut() = value;
    }
}

impl<V> Clone for FeedbackParameter<V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}





#[derive(Clone)]
pub struct FeedbackIter<I> where
    I: Iterator,
    I::Item : TraceData,
{
    parameter: FeedbackParameter<<I::Item as TraceData>::ValueType>,
    modifier: FeedbackFunctionType<I>,
    source: I,
}

impl<I> FeedbackIter<I> where
    I: Iterator,
    I::Item : TraceData,
{
    fn new(source : I, parameter : FeedbackValueType<I>, modifier : FeedbackFunctionType<I>) -> Self {
        Self {source, parameter: FeedbackParameter::<<I::Item as TraceData>::ValueType>::new(parameter), modifier }
    }
}

impl<I> Iterator for FeedbackIter<I>
where
    I: Iterator,
    I::Item : TraceData,
{
    type Item = (<I::Item as TraceData>::TimeType, <I::Item as TraceData>::ValueType, OptFeedParam<<I::Item as TraceData>::ValueType>);

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.source.next()?;
        let time = val.get_time();
        let value = (self.modifier)(&self.parameter.0.borrow(),&val.get_value());
        Some((time, value, Some(self.parameter.clone())))
    }
}

impl<X,Y,Z> TraceData for (X,Y,OptFeedParam<Z>) where X : Copy + Debug, Y: Clone + Debug, Z: Clone + Debug {
    type TimeType = X;
    type ValueType = Y;
    type ParameterType = Z;

    fn get_time(&self) -> Self::TimeType { self.0 }
    fn get_value(&self) -> &Self::ValueType { &self.1 }
    fn take_value(self) -> Self::ValueType { self.1 }

    fn get_parameter(&self) -> OptFeedParam<Z> { self.2.clone() }
}





pub trait FeedbackFilter<I> where
    I: Iterator,
    I::Item : TraceData
{
    fn feedback(self, modifier: FeedbackFunctionType<I>, parameter: FeedbackValueType<I>) -> FeedbackIter<I>;
}

impl<I> FeedbackFilter<I> for I where
    I: Iterator,
    I::Item : TraceData
{
    fn feedback(self, modifier: FeedbackFunctionType<I>, parameter: FeedbackValueType<I>) -> FeedbackIter<I> {
        FeedbackIter::new(self, parameter, modifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Real;
    use common::Intensity;

    #[test]
    fn sample_data_zero() {
        let input: Vec<Intensity> = vec![1, 6, 2, 1, 3, 1, 0];
        let output: Vec<Real> = input
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .feedback(|x,y|x + y,0.)
            .map(|(_, x, _)| x)
            .collect();

        assert_eq!(output[0], 1.);
        assert_eq!(output[1], 6.);
        assert_eq!(output[2], 2.);
        assert_eq!(output[3], 1.);
        assert_eq!(output[4], 3.);
        assert_eq!(output[5], 1.);
        assert_eq!(output[6], 0.);
    }

    #[test]
    fn sample_data_update() {
        let input: Vec<Intensity> = vec![1, 6, 2, 1, 3, 1, 0];
        let output: Vec<Real> = input
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .feedback(|x,y|x + y,0.)
            .map(|(_, x, m)| {
                if let Some(mm) = m { mm.set(2.); }
                x
            })
            .collect::<Vec<_>>();

        assert_eq!(output[0], 1.);
        assert_eq!(output[1], 8.);
        assert_eq!(output[2], 4.);
        assert_eq!(output[3], 3.);
        assert_eq!(output[4], 5.);
        assert_eq!(output[5], 3.);
        assert_eq!(output[6], 2.);
    }
}
