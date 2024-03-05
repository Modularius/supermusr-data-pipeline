use anyhow::{anyhow, Result};
use std::collections::{BTreeMap, HashMap};
use supermusr_common::Channel;
use crate::{base::{AnalysisKey, EventList}, message_group::{Header, MessageGroup}};

#[derive(Default)]
pub(crate) struct ChannelPairEventList {
    pub(crate) detected: EventList,
    pub(crate) simulated: EventList,
}

pub(crate) type ChannelList = HashMap<Channel, ChannelPairEventList>;

#[derive(Default)]
pub(crate) struct MessagePair {
    pub(crate) headers: Header,
    pub(crate) channels: ChannelList
}

impl MessagePair {
    pub(crate) fn from_message_group(message_group : &MessageGroup) -> Option<Result<Self>> {
        
        Option::zip(message_group.detected.as_ref(),message_group.simulated.as_ref())
            .map(|(detected, simulated)| {
                if detected.message.keys().collect::<Vec<_>>() != simulated.keys().collect::<Vec<_>>() {
                    return Err(anyhow!(
                        "Channel mismatch: {0:?}, {1:?}.",
                        detected.message.keys().collect::<Vec<_>>(),
                        simulated.keys().collect::<Vec<_>>()
                    ));
                }
                Ok(Self {
                    headers: detected.header.clone(),
                    channels: {
                        HashMap::from_iter(detected.message.iter().map(|(c, val)| {
                            (*c, ChannelPairEventList {
                                detected: (*val).clone(),
                                simulated: (*simulated.get(c).unwrap()).clone(),
                            })
                        }))
                    }
                })
            }
        )
    }
}

pub(crate) type MessagePairVector = Vec<MessagePair>;
pub(crate) type MessagePairVectorContainer = BTreeMap<AnalysisKey, MessagePairVector>;