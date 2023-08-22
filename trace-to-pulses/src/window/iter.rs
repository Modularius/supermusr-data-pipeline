use crate::{
    tracedata::{TraceData, TraceValue},
    trace_iterators::feedback::OptFeedParam,
};

use super::Window;


#[derive(Clone)]
pub struct WindowIter<I, W> where
    I: Iterator,
    I::Item : TraceData,
    W: Window,
{
    window: W,
    source: I,
}

impl<I, W> WindowIter<I, W> where
    I: Iterator,
    I::Item : TraceData,
    W: Window,
{
    pub fn new(source: I, window: W) -> Self {
        WindowIter { source, window}
    }
    #[cfg(test)]
    pub fn get_window(&self) -> &W {
        &self.window
    }
}

impl<I, W> Iterator for WindowIter<I, W> where
    I: Iterator,
    I::Item : TraceData,
    W: Window<
        TimeType = <I::Item as TraceData>::TimeType,
        InputType = <I::Item as TraceData>::ValueType
    >,
{
    type Item = (W::TimeType, W::OutputType, OptFeedParam<<<<I as Iterator>::Item as TraceData>::ValueType as TraceValue>::FeedbackType>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let val = self.source.next()?;
            if self.window.push(val.get_value().clone()) {
                return Some((self.window.apply_time_shift(val.get_time()), self.window.stats()?,val.get_parameter()));
            }
        }
    }
}
pub trait WindowFilter<I, W> where
    I: Iterator,
    I::Item : TraceData,
    W: Window,
{
    fn window(self, window: W) -> WindowIter<I, W>;
}

impl<I, W> WindowFilter<I, W> for I where
    I: Iterator,
    I::Item : TraceData,
    W: Window,
{
    fn window(self, window: W) -> WindowIter<I, W> {
        WindowIter::<I, W>::new(self, window)
    }
}
