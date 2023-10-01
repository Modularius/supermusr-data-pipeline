use std::{collections::VecDeque, slice::Iter};

use crate::{
    Real,
    RealArray,
    tracedata::{TraceData, TraceValue},
};
use num::integer::binomial;

use super::iter::{TraceIter, TraceIterType};

#[derive(Default, Clone)]
pub struct Memory<'a, T> where T : TraceData
{
    memory: Iter<'a,T>,
    source : Iter<'a,T>,
}

impl<'a,T> TraceIterType for Memory<'a,T> where T : TraceData {}

impl<'a,T> Memory<'a,T> where T : TraceData {
    pub fn new(memory: &'a Vec<T>) -> Self {
        Memory {
            memory: memory.iter(),
            source: Iter::default(),
        }
    }
    pub fn reset() {

    }
}

impl<'a, I> Iterator for TraceIter<Memory<'a, I::Item>,I> where
    I: Iterator,
    I::Item : TraceData,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.child.source.next() {
            Some(value) => Some(value.clone()),
            None => self.source.next(),
        }
    }
}





pub trait MemoryFilter<'a, I> where
    I: Iterator,
    I::Item : TraceData,
{
    fn memory(self, memory: &'a Vec<I::Item>) -> TraceIter<Memory<'a, I::Item>, I>;
}

impl<'a, I> MemoryFilter<'a, I> for I where
    I: Iterator,
    I::Item : TraceData,
{
    fn memory(self, memory: &'a Vec<I::Item>) -> TraceIter<Memory<'a, I::Item>, I> {
        TraceIter::new(Memory::new(memory), self)
    }
}



