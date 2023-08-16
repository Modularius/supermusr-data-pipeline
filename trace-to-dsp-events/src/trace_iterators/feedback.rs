use std::{fmt::Debug,cell::Cell, rc::Rc, iter::Map};

use crate::tracedata::TraceData;

type FeedbackValueType<I> = <<I as Iterator>::Item as TraceData>::ValueType;
type FeedbackFunctionType<I,P> = fn(&FeedbackValueType<I>, &P)->FeedbackValueType<I>;

#[derive(Default)]
pub struct FeedbackParameter<V>(pub Rc<Cell<V>>);
pub type OptFeedParam<V> = Option<FeedbackParameter<V>>;

impl<V> FeedbackParameter<V> {
    pub fn new(initial : V) -> Self {
        Self(Rc::new(Cell::new(initial)))
    }
    pub fn set(&self, value : V) {
        (*self.0).set(value);
    }
}

impl<V> Clone for FeedbackParameter<V> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}





#[derive(Clone)]
pub struct FeedbackIter<I,P> where
    I: Iterator,
    I::Item : TraceData,
{
    parameter: FeedbackParameter<P>,
    modifier: FeedbackFunctionType<I,P>,
    source: I,
}

impl<I,P> FeedbackIter<I,P> where
    I: Iterator,
    I::Item : TraceData,
{
    fn new(source : I, parameter : P, modifier : FeedbackFunctionType<I,P>) -> Self {
        Self {source, parameter: FeedbackParameter::<P>::new(parameter), modifier }
    }
}

impl<I,P> Iterator for FeedbackIter<I,P>
where
    I: Iterator,
    I::Item : TraceData,
    P : Copy + Debug
{
    type Item = (<I::Item as TraceData>::TimeType, <I::Item as TraceData>::ValueType, OptFeedParam<P>);

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.source.next()?;
        let time = val.get_time();
        let value = (self.modifier)(&val.get_value(),&self.parameter.0.get());//val.clone_value();//

        // LOG
        //log::info!("Applied correction of {0:?}", self.parameter.0.get());
        //let r = Rc::strong_count(&self.parameter.0);
        // LOG
        //log::info!("Number of references: {0:?}",r);
        Some((time, value, Some(self.parameter.clone())))//
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





pub trait FeedbackFilter<I,P> where
    I: Iterator,
    I::Item : TraceData,
{
    fn start_feedback(self, modifier: FeedbackFunctionType<I,P>) -> FeedbackIter<I,P>;
}

impl<I,P> FeedbackFilter<I,P> for I where
    I: Iterator,
    I::Item : TraceData,
    P : Default
{
    fn start_feedback(self, modifier: FeedbackFunctionType<I,P>) -> FeedbackIter<I,P> {
        FeedbackIter::new(self, P::default(), modifier)
    }
}


pub trait EndFeedbackFilter<I,X,Y,Z> where
    I: Iterator<Item = (X,Y,OptFeedParam<Z>)>,
{
    fn end_feedback(self) -> Map<I,fn((X,Y,OptFeedParam<Z>))->(X,Y)>;
}

impl<I,X,Y,Z> EndFeedbackFilter<I,X,Y,Z> for I where
    I: Iterator<Item = (X,Y,OptFeedParam<Z>)>,
{
    fn end_feedback(self) -> Map<I,fn((X,Y,OptFeedParam<Z>))->(X,Y)> {
        self.map(|(x,y,_)|(x,y))
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
            .start_feedback(|x,&y : &Real|x + y)
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
            .start_feedback(|x,&y : &Real|x + y)
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
