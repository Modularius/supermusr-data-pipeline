
use std::collections::VecDeque;

use crate::{Real, event_iterators::pulse_formation::Gaussian};
pub type RealArray<const N: usize> = [Real; N];

pub trait Modifier {
    type TimeType: Copy;
    type ValueType: Copy;

    fn modify(&mut self, time : Self::TimeType, value: Self::ValueType) -> Self::ValueType;
}

struct PulseCorrector {
    pulses: VecDeque<Gaussian>,
}

impl Modifier for PulseCorrector {
    type TimeType = Real;
    type ValueType = Real;

    fn modify(&mut self, time : Self::TimeType, value: Self::ValueType) -> Self::ValueType
    {
        if self.pulses.front().map(|gaussian|gaussian.get_value_at(time) < 1e-5).unwrap_or(false) {
            self.pulses.pop_front();
        }
        value - self.pulses.iter().map(|gaussian|gaussian.get_value_at(time)).sum::<Real>()
    }
}


pub struct ModifierIter<'a, I,M> where
    I: Iterator<Item = (Real, Real)>,
    M: Modifier<TimeType = Real, ValueType = Real> + 'a,
{
    source: I,
    modifier : &'a mut M,
}

impl<'a, I,M> ModifierIter<'a, I,M> where
    I: Iterator<Item = (Real, Real)>,
    M: Modifier<TimeType = Real, ValueType = Real> + 'a,
{
    pub fn new(source: I, modifier : &'a mut M) -> Self {
        ModifierIter { source, modifier }
    }
}
pub trait ModifierFilter<'a, I,M> where
    I: Iterator<Item = (Real, Real)>,
    M: Modifier<TimeType = Real, ValueType = Real> + 'a,
{
    fn modify(self, modifier : &'a mut M) -> ModifierIter<'a, I,M>;
}

impl<'a, I,M> ModifierFilter<'a, I,M> for I where
    I: Iterator<Item = (Real, Real)>,
    M: Modifier<TimeType = Real, ValueType = Real> + 'a,
{
    fn modify(self, modifier : &'a mut M) -> ModifierIter<'a, I,M> {
        ModifierIter::new(self, modifier)
    }
}