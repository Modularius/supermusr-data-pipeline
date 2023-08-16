use crate::{
    detectors::pulse_detector::PulseEvent,
    Real,
    tracedata::TraceData,
};


fn sum_event_energy_at<'a>(events : &'a [PulseEvent], time: Real) -> Real {
    let sum = events.iter()
        .map(|event|
            if event.get_data()
                .get_standard_deviation()
                .unwrap_or_default() > 0.
            {
                event.get_data().get_effective_value_at(time)
            } else {
                0.
            }
        ).sum::<Real>();
    sum
}




#[derive(Clone)]
pub struct SimulationIter<'a, I> where
    I: Iterator,
    I::Item : TraceData<TimeType = Real, ValueType = Real>,
{
    source : I,
    events : &'a [PulseEvent],
}


impl<'a,I> Iterator for SimulationIter<'a,I> where
    I: Iterator,
    I::Item : TraceData<TimeType = Real, ValueType = Real>,
{
    type Item = (Real,Real);

    fn next(&mut self) -> Option<Self::Item> {
        let trace = self.source.next()?;
        Some((
            trace.get_time(),
            sum_event_energy_at(self.events,trace.get_time()),
        ))
    }
}





#[derive(Clone)]
pub struct EvaluationIter<'a, I> where
    I: Iterator,
    I::Item : TraceData<TimeType = Real, ValueType = Real>,
{
    source : I,
    events : &'a [PulseEvent],
}
impl<'a,I> Iterator for EvaluationIter<'a,I> where
    I: Iterator,
    I::Item : TraceData<TimeType = Real, ValueType = Real>,
{
    type Item = (Real,Real,Real);

    fn next(&mut self) -> Option<Self::Item> {
        let trace = self.source.next()?;
        Some((
            trace.get_time(),
            trace.clone_value(),
            (trace.get_value() - sum_event_energy_at(self.events,trace.get_time())).abs(),
        ))
    }
}




pub trait ToTrace<'a,I> where
    I: Iterator<Item = (Real,Real)>,
{
    fn to_trace(self, events : &'a [PulseEvent]) -> SimulationIter<'a,I>;
    fn evaluate_events(self, events : &'a [PulseEvent]) -> EvaluationIter<'a,I>;
}

impl<'a,I> ToTrace<'a,I> for I where
    I: Iterator<Item = (Real,Real)> + Clone,
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

