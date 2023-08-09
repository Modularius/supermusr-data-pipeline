use std::collections::VecDeque;

use super::RealArray;
use crate::Real;
use num::integer::binomial;

#[derive(Clone)]
pub struct CombineIter<I,V> where
    I: Iterator<Item = (Real, V)>,
{
    warm_up: usize,
    baseline: Real,
    source: I,
}

impl<I> FindBaselineIter<I> where
    I: Iterator<Item = (Real, Real)>,
{
    pub fn new(source: I, warm_up : usize) -> Self {
        FindBaselineIter { source, warm_up, baseline: Real::MAX }
    }
}

impl<I> Iterator for FindBaselineIter<I>
where
    I: Iterator<Item = (Real, Real)>,
{
    type Item = (Real, Real);

    fn next(&mut self) -> Option<Self::Item> {
        while self.warm_up > 0 {
            match self.source.next() {
                Some((_,v)) => self.baseline = Real::min(self.baseline, v),
                None => return None,
            }
            self.warm_up -= 1;
            if self.warm_up == 0 { log::info!("{0}",self.baseline); }
        }
        self.source.next()
            .map(|(i,v)|(i,v - self.baseline))
    }
}

pub trait FindBaselineFilter<I> where
    I: Iterator<Item = (Real, Real)>,
{
    fn find_baseline(self, warm_up : usize) -> FindBaselineIter<I>;
}

impl<I> FindBaselineFilter<I> for I where
    I: Iterator<Item = (Real, Real)>,
{
    fn find_baseline(self, warm_up : usize) -> FindBaselineIter<I> {
        FindBaselineIter::new(self, warm_up)
    }
}

#[cfg(test)]
mod tests {
    use super::{FindBaselineFilter, Real, RealArray};
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
