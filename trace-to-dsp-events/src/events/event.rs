use crate::Real;
use std::fmt::Debug;
use std::fmt::Display;

pub trait EventData: Default + Debug + Clone + Display {
    fn make_event(self, time : Real) -> Event<Self> {
        Event::<Self> { time, data: self }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Event<D> where
    D: EventData,
{
    pub time: Real,
    pub data: D,
}
impl<D> Event<D> where D: EventData,
{
    pub fn new(time: Real, data: D) -> Self {
        Self { time, data }
    }
    pub fn get_time(&self) -> Real {
        self.time
    }
    pub fn get_data(&self) -> &D {
        &self.data
    }

    pub fn get_data_mut(&mut self) -> &mut D {
        &mut self.data
    }

    pub fn take_data(self) -> D {
        self.data
    }
}

impl<D> PartialEq for Event<D> where D: EventData {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl<C> Display for Event<C>
where
    C: EventData,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0},{1};", self.time, self.data))
    }
}