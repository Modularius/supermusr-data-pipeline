use std::fmt::Display;

use crate::events::Event;
use crate::tracedata::EventData;
use crate::{Detector, Real};

#[derive(Default, Debug, Clone,PartialEq)]
pub enum LocalExtremumClass {
    #[default]
    LocalMax,
    LocalMin,
}

#[derive(Default, Debug, Clone)]
pub struct LocalExtremumData {
    class: LocalExtremumClass,
    value : Option<Real>,
}
impl LocalExtremumData {
    pub fn new(class : LocalExtremumClass) -> Self {
        LocalExtremumData{ class, value: None }
    }
    pub fn with_value(class : LocalExtremumClass, value : Real) -> Self {
        LocalExtremumData{ class, value: Some(value) }
    }
    pub fn get_class(&self) -> LocalExtremumClass { self.class.clone() }
    pub fn get_value(&self) -> Option<Real> { self.value }
}
impl EventData for LocalExtremumData {}

impl Display for LocalExtremumData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0},{1}", self.value.unwrap_or(0.),
            match self.class { LocalExtremumClass::LocalMax => 1, LocalExtremumClass::LocalMin => -1, })
        )
    }
}

type LocalExtremumEvent = Event<Real, LocalExtremumData>;







#[derive(Default,Clone)]
pub struct LocalExtremumDetector {
    prev: Option<(Real,Option<Real>)>,
}

impl LocalExtremumDetector {
    pub fn new() -> Self { Self::default() }
}

impl Detector for LocalExtremumDetector {
    type TimeType = Real;
    type ValueType = Real;
    type DataType = LocalExtremumData;

    fn signal(&mut self, time: Real, value: Real) -> Option<LocalExtremumEvent> {
        if let Some((prev_value,Some(prev_prev_value))) = self.prev {
            let return_value = {
                if (prev_prev_value < prev_value && prev_value >= value) || (prev_prev_value <= prev_value && prev_value > value) {
                    Some(
                        LocalExtremumData::with_value(LocalExtremumClass::LocalMax,prev_value)
                        .make_event(time - 1.)
                    )
                } else if (prev_prev_value > prev_value && prev_value <= value) || (prev_prev_value >= prev_value && prev_value < value) {
                    Some(
                        LocalExtremumData::with_value(LocalExtremumClass::LocalMin, prev_value)
                        .make_event(time - 1.)
                    )
                } else { None }
            };
            self.prev = Some((value,Some(prev_value)));
            return_value       
        } else if let Some((prev_value,None)) = self.prev {
            self.prev = Some((value,Some(prev_value)));
            None
        }
        else {
            self.prev = Some((value,None));
            None
        }
    }
}





#[derive(Default, Debug, Clone)]
pub struct PeakData {
    value : Option<Real>,
    time_since_start : Option<Real>,
    time_till_end : Option<Real>,
}
impl PeakData {
    pub fn get_value(&self) -> Option<Real> { self.value }
    pub fn get_time_since_start(&self) -> Option<Real> { self.time_since_start }
    pub fn get_time_till_end(&self) -> Option<Real> { self.time_till_end }
}
impl EventData for PeakData {}

impl Display for PeakData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0},{1},{2}", self.value.unwrap_or(0.), self.time_since_start.unwrap_or(0.), self.time_till_end.unwrap_or(0.)))
    }
}

type PeakEvent = Event<Real, PeakData>;


pub fn local_extrema_to_peaks((left,mid,right) : (LocalExtremumEvent,LocalExtremumEvent,LocalExtremumEvent)) -> Option<PeakEvent> {
    if mid.get_data().get_class() == LocalExtremumClass::LocalMax {
        Some(
            PeakData {
                value: mid.get_data().get_value(),
                time_since_start: Some(mid.get_time() - left.get_time()),
                time_till_end: Some(right.get_time() - mid.get_time()),
            }.make_event(mid.get_time())
        )
    } else {
        None
    }
}


/*

#[derive(Default,Clone)]
pub struct PeakDetector {
    detector : LocalExtremeDetector,
    prev: VecDeque<LocalExtremeEvent>,
}

impl PeakDetector {
    pub fn new() -> Self { PeakDetector { prev: VecDeque::<LocalExtremeEvent>::with_capacity(3), ..Default::default() } }
}

impl Detector for PeakDetector {
    type TimeType = Real;
    type ValueType = Real;
    type DataType = PeakData;

    fn signal(&mut self, time: Real, value: Real) -> Option<PeakEvent> {
        let event = self.detector.signal(time,value);
        if let Some(event) = event {
            self.prev.push_front(event);
        }
        let return_value = if self.prev.len() > 2 {
            two_sided(self.prev.get(0), self.prev.get(1), self.prev.get(2))
        } else {
            None
        };
        self.prev.truncate(2);
        return_value
    }
}*/





#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::processing;
    use super::*;

    #[test]
    fn zero_data() {
        let data = [];
        let mut detector = LocalExtremumDetector::new();
        let results = data.iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .map(|(i,v)|detector.signal(i,v))
            .collect_vec();

        assert!(results.is_empty());
    }

    #[test]
    fn test_gate_zero_threshold() {
        let data = [4, 3, 2, 5, 6, 1, 5, 7, 2, 4];
        let mut detector = LocalExtremumDetector::new();
        let results = data.iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .map(|(i,v)|detector.signal(i,v))
            .collect_vec();

            assert_eq!(results.len(),data.len());
            assert_eq!(results[0],None);
            assert_eq!(results[1],None);
            assert_eq!(results[2],None);
            assert_eq!(results[3],Some(LocalExtremumData::with_value(LocalExtremumClass::LocalMin, 2.).make_event(2.)));
            assert_eq!(results[4],None);
            assert_eq!(results[5],Some(LocalExtremumData::with_value(LocalExtremumClass::LocalMax, 6.).make_event(4.)));
            assert_eq!(results[6],Some(LocalExtremumData::with_value(LocalExtremumClass::LocalMin, 1.).make_event(5.)));
            assert_eq!(results[7],None);
            assert_eq!(results[8],Some(LocalExtremumData::with_value(LocalExtremumClass::LocalMax, 7.).make_event(7.)));
            assert_eq!(results[9],Some(LocalExtremumData::with_value(LocalExtremumClass::LocalMin, 2.).make_event(8.)));
    }
}