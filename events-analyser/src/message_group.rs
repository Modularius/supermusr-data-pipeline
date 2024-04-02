use crate::base::{EventList, MessageKey};
use rdkafka::message::{BorrowedMessage, Headers, Message};
use std::collections::{BTreeMap, HashMap};
use supermusr_common::{tracer::Spanned, Channel};
use supermusr_streaming_types::dev1_digitizer_event_v1_generated::DigitizerEventListMessage;

pub(crate) type SimulatedMessage = ChannelEventList;
#[derive(Debug)]
pub(crate) struct DetectedMessage {
    pub(crate) header: Header,
    pub(crate) message: ChannelEventList,
}

#[derive(Default)]
pub(crate) struct MessageGroup {
    pub(crate) detected: Option<Spanned<DetectedMessage>>,
    pub(crate) simulated: Option<Spanned<SimulatedMessage>>,
}

pub(crate) type MessageGroupContainer = BTreeMap<MessageKey, MessageGroup>;

pub(crate) trait MessageExtractable<'a> {
    type MessageType;
    fn from_message(m: &Self::MessageType) -> Self;
}

pub(crate) type ChannelEventList = HashMap<Channel, EventList>;
impl<'a> MessageExtractable<'a> for ChannelEventList {
    type MessageType = DigitizerEventListMessage<'a>;

    fn from_message(message: &DigitizerEventListMessage<'a>) -> Self {
        let mut list = ChannelEventList::new();
        for (i, c) in message.channel().unwrap().iter().enumerate() {
            let event_list = list.entry(c).or_default();
            event_list.time.push(message.time().unwrap().get(i));
            event_list.voltage.push(message.voltage().unwrap().get(i));
        }
        list
    }
}

pub(crate) type Header = HashMap<String, String>;
impl<'a> MessageExtractable<'a> for Header {
    type MessageType = BorrowedMessage<'a>;

    fn from_message(m: &BorrowedMessage<'a>) -> Self {
        let mut list = Header::new();
        if let Some(headers) = m.headers().map(|h| h.detach()) {
            for h in headers.iter() {
                let val = String::from_utf8(h.value.unwrap().to_owned()).unwrap();
                list.insert(h.key.to_owned(), val);
            }
        };
        list
    }
}
