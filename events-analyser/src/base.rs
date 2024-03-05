use chrono::{DateTime, Utc};
use supermusr_common::{DigitizerId, FrameNumber, Intensity, Time};
use supermusr_streaming_types::dev1_digitizer_event_v1_generated::DigitizerEventListMessage;


// HashKeys

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AnalysisKey {
    pub(crate) digitiser_id: DigitizerId,
    pub(crate) frame_number: FrameNumber,
}

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct MessageKey {
    pub(crate) ts: DateTime<Utc>,
    pub(crate) analysis_key: AnalysisKey,
}

impl MessageKey {
    pub(crate) fn new(thing: &DigitizerEventListMessage) -> Self {
        MessageKey {
            ts: (*thing.metadata().timestamp().unwrap()).into(),
            analysis_key: AnalysisKey {
                digitiser_id: thing.digitizer_id(),
                frame_number: thing.metadata().frame_number(),
            }
        }
    }
}


#[derive(Default, Clone)]
pub(crate) struct EventList {
    pub(crate) voltage: Vec<Intensity>,
    pub(crate) time: Vec<Time>,
}