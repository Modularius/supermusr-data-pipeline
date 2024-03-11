use crate::{
    base::{AnalysisKey, EventList},
    message_group::{Header, MessageGroup},
};
use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};
use supermusr_common::Channel;

#[derive(Default)]
pub(crate) struct ChannelPairEventList {
    pub(crate) detected: EventList,
    pub(crate) simulated: EventList,
}

/*fn dist_sq(a : (&Time,&Intensity), b : (&Time,&Intensity)) -> f64 {
    f64::powi(*b.0 as f64 - *a.0 as f64,2) + f64::powi(*b.1 as f64 - *a.1 as f64,2)
}
use supermusr_common::{Channel, Intensity, Time};

impl ChannelPairEventList {
    fn calc_quality(&self) -> Vec<f64> {
        let mut detected_iter = Iterator::zip(self.detected.time.iter(), self.detected.voltage.iter()).enumerate().peekable();
        let mut count = vec![0.0; self.detected.time.len()];

        for current_simulated in Iterator::zip(self.simulated.time.iter(), self.simulated.voltage.iter()) {
            let (current_detected_index, current_detected) = detected_iter.next().unwrap();
            let dist_sq_to_current = dist_sq(current_detected,current_simulated);

            if let Some(&(next_detected_index,next_detected)) = detected_iter.peek() {
                let dist_sq_to_next = dist_sq(next_detected,current_simulated);

                if dist_sq_to_current < dist_sq_to_next {
                    count[current_detected_index] += dist_sq_to_current;
                } else {
                    count[next_detected_index] += dist_sq_to_next;
                }
            } else {
                count[current_detected_index] += dist_sq_to_current;
            };
        }
        count
    }
}*/

pub(crate) type ChannelPairList = HashMap<Channel, ChannelPairEventList>;

// This could be refactored to eliminate the need to store the events possibly
#[derive(Default)]
pub(crate) struct MessagePair {
    pub(crate) headers: Header,
    pub(crate) channels: ChannelPairList,
}

impl MessagePair {
    pub(crate) fn from_message_group(message_group: &MessageGroup) -> Option<Result<Self>> {
        Option::zip(
            message_group.detected.as_ref(),
            message_group.simulated.as_ref(),
        )
        .map(|(detected, simulated)| {
            {
                let detected_keys = detected.message.keys().collect::<HashSet<_>>();
                let simulated_keys = simulated.keys().collect::<HashSet<_>>();
                if detected_keys != simulated_keys {
                    return Err(anyhow!(
                        "Channel mismatch: {0:?}, {1:?}.",
                        detected,
                        simulated
                    ));
                }
            }
            Ok(Self {
                headers: detected.header.clone(),
                channels: {
                    HashMap::from_iter(detected.message.iter().map(|(c, val)| {
                        (
                            *c,
                            ChannelPairEventList {
                                detected: (*val).clone(),
                                simulated: (*simulated.get(c).unwrap()).clone(),
                            },
                        )
                    }))
                },
            })
        })
    }
}

pub(crate) type MessagePairVector = Vec<MessagePair>;
pub(crate) type MessagePairVectorContainer = HashMap<AnalysisKey, MessagePairVector>;
