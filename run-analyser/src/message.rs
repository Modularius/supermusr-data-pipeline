use rdkafka::message::{BorrowedMessage, Headers, Message};
use std::collections::{BTreeMap, HashMap, HashSet};
use supermusr_common::{Channel, Intensity, Time};
use supermusr_streaming_types::{aev1_frame_assembled_event_v1_generated::FrameAssembledEventListMessage, dev1_digitizer_event_v1_generated::DigitizerEventListMessage};

#[derive(Default)]
pub(crate) struct Pair<T : Default> {
    pub(crate) detected : T,
    pub(crate) simulated : T,
}

pub(crate) type ByChannel<T> = HashMap<Channel,T>;

#[derive(Default, Debug, Clone)]
pub(crate) struct EventList {
    pub(crate) voltage: Vec<Intensity>,
    pub(crate) time: Vec<Time>,
}

pub(crate) type PairOfEventListByChannel = HashMap<Channel,Pair<EventList>>;