use std::marker::PhantomData;

use crate::Real;

use super::Window;


pub trait Realisable : From<Real> + Default {}
impl <T> Realisable for T where T : From<Real> + Default {}

#[derive(Default, Clone, Copy)]
pub struct TrivialWindow<O> where O : Realisable {
    value: Real,
    phantom: PhantomData<O>,
}
impl<O> Window for TrivialWindow<O> where O : Realisable {
    type TimeType = Real;
    type InputType = Real;
    type OutputType = O;

    fn push(&mut self, value: Real) -> bool {
        self.value = value;
        true
    }
    fn stats(&self) -> Option<Self::OutputType> {
        Some(O::from(self.value))
    }
    fn apply_time_shift(&self, time : Real) -> Real { time }
}
