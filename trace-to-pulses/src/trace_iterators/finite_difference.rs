use std::collections::VecDeque;

use crate::{
    Real,
    RealArray,
    tracedata::TraceData,
};
use num::integer::binomial;

#[derive(Clone)]
pub struct FiniteDifferencesIter<I, const N: usize> where
    I: Iterator,
    I::Item : TraceData<TimeType = Real, ValueType = Real>,
{
    coefficients: Vec<Vec<Real>>,
    values: VecDeque<Real>,
    source: I,
}

impl<I, const N: usize> FiniteDifferencesIter<I, N> where
    I: Iterator,
    I::Item : TraceData<TimeType = Real, ValueType = Real>,
{
    pub fn new(source: I) -> Self {
        FiniteDifferencesIter {
            source,
            values: VecDeque::<Real>::with_capacity(N),
            coefficients: (0..N)
                .map(|n| {
                    (0..=n)
                        .map(|k| (if k & 1 == 1 { -1. } else { 1. }) * (binomial(n, k) as Real))
                        .collect()
                })
                .collect(),
        }
    }
}

impl<I, const N: usize> FiniteDifferencesIter<I, N> where
    I: Iterator,
    I::Item : TraceData<TimeType = Real, ValueType = Real>,
{
    fn nth_difference(&self, n: usize) -> Real {
        (0..=n)
            .map(|k| self.coefficients[n][k] * self.values[k])
            .sum()
    }
    fn next_no_index(&mut self, value: Real) -> RealArray<N> {
        self.values.push_front(value);
        let mut diffs = [Real::default(); N];
        for n in 0..N {
            diffs[n] = self.nth_difference(n);
        }
        self.values.pop_back();
        diffs
    }
}

impl<I, const N: usize> Iterator for FiniteDifferencesIter<I, N> where
    I: Iterator,
    I::Item : TraceData<TimeType = Real, ValueType = Real>,
{
    type Item = (Real, RealArray<N>);

    fn next(&mut self) -> Option<Self::Item> {
        let mut value = self.source.next();
        while self.values.len() + 1 < self.values.capacity() {
            match &value {
                Some(trace) => {
                    self.values.push_front(trace.get_value().clone());
                    value = self.source.next();
                }
                None => return None,
            }
        }
        let trace = value?;
        Some((trace.get_time(),self.next_no_index(trace.take_value())))
    }
}

pub trait FiniteDifferencesFilter<I, const N: usize> where
    I: Iterator,
    I::Item : TraceData<TimeType = Real, ValueType = Real>,
{
    fn finite_differences(self) -> FiniteDifferencesIter<I, N>;
}

impl<I, const N: usize> FiniteDifferencesFilter<I, N> for I where
    I: Iterator,
    I::Item : TraceData<TimeType = Real, ValueType = Real>,
{
    fn finite_differences(self) -> FiniteDifferencesIter<I, N> {
        FiniteDifferencesIter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{FiniteDifferencesFilter, Real, RealArray};
    use common::Intensity;

    #[test]
    fn sample_data() {
        let input: Vec<Intensity> = vec![0, 6, 2, 1, 3, 1, 0];
        let output: Vec<RealArray<3>> = input
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .finite_differences()
            .map(|(_, x)| x)
            .collect();

        assert_eq!(output[0], [2., -4., -10.]);
        assert_eq!(output[1], [1., -1., 3.]);
        assert_eq!(output[2], [3., 2., 3.]);
        assert_eq!(output[3], [1., -2., -4.]);
        assert_eq!(output[4], [0., -1., 1.]);
    }
}
