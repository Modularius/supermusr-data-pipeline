use std::collections::HashMap;
use supermusr_common::{Channel, Intensity, Time};

#[derive(Default)]
pub(crate) struct Pair<T : Default> {
    pub(crate) detected : T,
    pub(crate) simulated : T,
}

pub(crate) type ByChannel<T> = HashMap<Channel,T>;

#[derive(Default, Debug, Clone)]
pub(crate) struct EventList {
    pub(crate) frames: usize,
    pub(crate) voltage: Vec<Intensity>,
    pub(crate) time: Vec<Time>,
}

pub(crate) type PairOfEventListByChannel = ByChannel<Pair<EventList>>;