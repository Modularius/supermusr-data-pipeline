use std::{marker::PhantomData, ops::Add};

use crate::Real;

pub mod composite;
pub mod gate;
pub mod noise_smoothing_window;
pub mod smoothing_window;
pub mod trivial;


pub trait Window {
    type InputType: Copy;
    type OutputType;

    fn push(&mut self, value: Self::InputType) -> bool;
    fn stats(&self) -> Option<Self::OutputType>;
    fn get_time_shift(&self) -> Real;
}

#[derive(Clone)]
pub struct WindowIter<I, W>
where
    I: Iterator<Item = (Real, W::InputType)>,
    W: Window,
{
    window: W,
    source: I,
    delta: Option<(W::InputType,fn(W::InputType,W::InputType)->W::InputType)>,
}

impl<I, W> WindowIter<I, W>
where
    I: Iterator<Item = (Real, W::InputType)>,
    W: Window,
{
    pub fn new(source: I, window: W) -> Self {
        WindowIter { source, window, delta:None }
    }
    pub fn set_delta(&mut self, delta : W::InputType) {
        self.delta.map(|(mut x,_)|{x = delta;});
    }
    pub fn set_delta_op(&mut self, delta_op : fn(W::InputType,W::InputType)->W::InputType) {
        self.delta.map(|(_,mut x)|{x = delta_op;});
    }
    #[cfg(test)]
    pub fn get_window(&self) -> &W {
        &self.window
    }
}

impl<I, W> Iterator for WindowIter<I, W>
where
    I: Iterator<Item = (Real, W::InputType)>,
    W: Window,
{
    type Item = (Real, W::OutputType);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let val = match self.delta {
                Some((delta,delta_op)) => { let (time,value) = self.source.next()?; (time,delta_op(value,delta)) },
                None => self.source.next()?,
            };
            if self.window.push(val.1) {
                return Some((val.0 - self.window.get_time_shift(), self.window.stats()?));
            }
        }
    }
}
pub trait WindowFilter<I, W>
where
    W: Window,
    I: Iterator<Item = (Real, W::InputType)>,
{
    fn window(self, window: W) -> WindowIter<I, W>;
}

impl<I, W> WindowFilter<I, W> for I
where
    W: Window,
    I: Iterator<Item = (Real, W::InputType)>,
{
    fn window(self, window: W) -> WindowIter<I, W> {
        WindowIter::<I, W>::new(self, window)
    }
}





