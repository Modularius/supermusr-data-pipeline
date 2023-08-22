/// This find the baseline of a trace stream by reading through the first warm_up values
/// and setting the baseline to the minimum of these. From hereon, all subsequent values
/// 
use crate::{
    Real,
    tracedata::TraceData,
};

#[derive(Clone)]
pub struct FindBaselineIter<I> where
    I: Iterator,
    I::Item : TraceData
{
    warm_up: usize,
    baseline: Real,
    source: I,
}

impl<I> FindBaselineIter<I> where
    I: Iterator,
    I::Item : TraceData
{
    pub fn new(source: I, warm_up : usize) -> Self {
        FindBaselineIter { source, warm_up, baseline: Real::MAX }
    }
}

impl<I> Iterator for FindBaselineIter<I>
where
    I: Iterator,
    I::Item : TraceData<ValueType = Real>
{
    type Item = (<I::Item as TraceData>::TimeType, Real);

    fn next(&mut self) -> Option<Self::Item> {
        while self.warm_up > 0 {
            match self.source.next() {
                Some(trace) => self.baseline = Real::min(self.baseline, trace.take_value()),
                None => return None,
            }
            self.warm_up -= 1;
            //if self.warm_up == 0 { log::info!("{0}",self.baseline); }
        }
        self.source.next()
            .map(|trace|(trace.get_time(),trace.take_value() - self.baseline))
    }
}

pub trait FindBaselineFilter<I> where
    I: Iterator,
    I::Item : TraceData
{
    fn find_baseline(self, warm_up : usize) -> FindBaselineIter<I>;
}

impl<I> FindBaselineFilter<I> for I where
    I: Iterator,
    I::Item : TraceData
{
    fn find_baseline(self, warm_up : usize) -> FindBaselineIter<I> {
        FindBaselineIter::new(self, warm_up)
    }
}

#[cfg(test)]
mod tests {
    use super::{FindBaselineFilter, Real};
    use common::Intensity;

    #[test]
    fn sample_data() {
        let input: Vec<Intensity> = vec![1, 6, 2, 1, 3, 1, 0];
        let output: Vec<Real> = input
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .find_baseline(3)
            .map(|(_, x)| x)
            .collect();

        assert_eq!(output[0], 0.);
        assert_eq!(output[1], 2.);
        assert_eq!(output[2], 0.);
        assert_eq!(output[3], -1.);
    }
}
