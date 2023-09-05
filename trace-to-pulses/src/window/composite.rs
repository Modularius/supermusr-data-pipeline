use crate::{RealArray, Real};
use core::array::from_fn;

use crate::window::Window;

use super::trivial::{TrivialWindow, Realisable};

type BoxedDynWindow<O> = Box<dyn Window<TimeType = Real, InputType = Real, OutputType = O>>;

pub struct CompositeWindow<const N: usize, O> where O : Realisable {
    windows: [BoxedDynWindow<O>; N],
}
impl<const N: usize, O> CompositeWindow<N, O> where O : Realisable + 'static {
    pub fn new(windows: [BoxedDynWindow<O>; N]) -> Self {
        CompositeWindow { windows }
    }

    pub fn trivial() -> Self {
        CompositeWindow {
            windows: from_fn(|_| Box::new(TrivialWindow::<O>::default()) as BoxedDynWindow<O>),
        }
    }
}
impl<const N: usize, O> Window for CompositeWindow<N, O> where O : Realisable {
    type TimeType = Real;
    type InputType = RealArray<N>;
    type OutputType = [O; N];

    fn push(&mut self, value: RealArray<N>) -> bool {
        let mut full = true;
        for i in 0..N {
            full = full && self.windows[i].push(value.0[i])
        }
        full
    }
    fn stats(&self) -> Option<Self::OutputType> {
        // This will do for now, but calling stats twice is inefficient (maybe)
        if self.windows.iter().any(|window| window.stats().is_none()) {
            None
        } else {
            Some(from_fn(|i| self.windows[i].stats().unwrap()))
        }
    }
    fn apply_time_shift(&self, time : Real) -> Real { time }
}
