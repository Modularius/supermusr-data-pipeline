use crate::{
    events::EventData,
    event_iterators::pulse_formation::{
        PulseEvent,
        PulseData,
    },
    Detector, EventIter, Real,
};

pub struct SimulationIter<'a, I> where
    I: Iterator<Item = (Real,Real)>
{
    source : I,
    events : &'a [PulseEvent],
}
impl<'a,I> Iterator for SimulationIter<'a,I> where
    I: Iterator<Item = (Real,Real)>,
{
    type Item = (Real,Real);

    fn next(&mut self) -> Option<Self::Item> {
        let (time,_) = self.source.next()?;
        Some((time,self.events.iter().map(|event|event.get_data().get_intensity_at(time)).sum::<Real>()))
    }
}





pub struct EvaluationIter<'a, I> where
    I: Iterator<Item = (Real,Real)>
{
    source : I,
    events : &'a [PulseEvent],
}
impl<'a,I> Iterator for EvaluationIter<'a,I> where
    I: Iterator<Item = (Real,Real)>,
{
    type Item = (Real,Real,Real);

    fn next(&mut self) -> Option<Self::Item> {
        let (time,value) = self.source.next()?;
        Some((time,value,(value - self.events.iter().map(|event| {
            if event.get_data().get_radius().unwrap_or_default() > 0. {
                event.get_data().get_intensity_at(time)
            } else {
                0.
            }
        }).sum::<Real>()).abs()))
    }
}




pub trait ToTrace<'a,I> where
    I: Iterator<Item = (Real,Real)>,
{
    fn to_trace(self, events : &'a [PulseEvent]) -> SimulationIter<'a,I>;
    fn evaluate_events(self, events : &'a [PulseEvent]) -> EvaluationIter<'a,I>;
}

impl<'a,I> ToTrace<'a,I> for I where
    I: Iterator<Item = (Real,Real)>,
{
    fn to_trace(self, events : &'a [PulseEvent]) -> SimulationIter<'a,I> {
        SimulationIter {
            source: self,
            events
        }
    }
    fn evaluate_events(self, events : &'a [PulseEvent]) -> EvaluationIter<'a,I> {
        EvaluationIter {
            source: self,
            events
        }
    }
}
