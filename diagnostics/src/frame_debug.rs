use std::collections::HashMap;

use crate::FrameOpts;

use anyhow::Result;
use chrono::{DateTime, Utc};
use rdkafka::{
    consumer::{CommitMode, Consumer, StreamConsumer},
    Message,
};
use supermusr_common::{DigitizerId, FrameNumber};
use supermusr_streaming_types::{dat2_digitizer_analog_trace_v2_generated::{digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message}, dev2_digitizer_event_v2_generated::{
    digitizer_event_list_message_buffer_has_identifier, root_as_digitizer_event_list_message,
}};
use tracing::{debug, warn};

struct MessageData {
    id: DigitizerId,
    timestamp: Option<DateTime<Utc>>,
    period_number: u64,
    protons_per_pulse: u8,
    veto_flags: u16,
    running: bool,
}

fn is_metadata_equal(data: &[MessageData]) -> bool {
    let (_, eq) : (Option<&MessageData>, bool) = data.iter().fold((None,true), |prev, this| match prev {
        (_,false) => (None,false),
        (None,true) => (Some(this), true),
        (Some(prev),true) => {
            (Some(this),
                prev.timestamp == this.timestamp &&
                prev.period_number == this.period_number &&
                prev.protons_per_pulse == this.protons_per_pulse &&
                prev.running == this.running
            )
        }
    });
    eq
}

impl MessageData {
    fn println(&self) {
        println!(
            "  ---  {0:<2} {1:<38} p: {2:<8} c: {3:<8} r: {4:<8} v: {5:<8}",
            self.id,
            self.timestamp
                .map(|ts| ts.to_string())
                .unwrap_or(Default::default()),
            self.period_number,
            self.protons_per_pulse,
            self.running,
            self.veto_flags,
        )
    }
}

fn dump(frames : &HashMap<FrameNumber, (Vec<MessageData>,Vec<MessageData>)>) {
    let mut keys : Vec<FrameNumber> = Vec::from_iter(frames.keys().map(|x|*x));
    keys.sort();
    
    for key in keys {
        let vec1 = &frames[&key].0;
        let vec2 = &frames[&key].1;
        if vec1.len() != 7 {
            println!("{0} Incomplete Frame (Event List Messages)", key);
            vec1.iter().for_each(|m|m.println());
        } else if vec2.len() != 7 {
            println!("{0} Incomplete Frame (Trace Messages)", key);
            vec2.iter().for_each(|m|m.println());
        }
        else {
            if !is_metadata_equal(vec1.as_slice()) {
                println!("{0} Complete but Unequal Metadata (Event List Messages)", key);
                vec1.iter().for_each(|m|m.println());
            } else if !is_metadata_equal(vec2.as_slice()) {
                println!("{0} Complete but Unequal Metadata (Trace Messages)", key);
                vec2.iter().for_each(|m|m.println());
            } else {
                let f = &vec1[0];
                println!("{0:<10}{1:<38} p: {2:<8} c: {3:<8} r: {4:<8}",
                    key,
                    f.timestamp.map(|ts|ts.to_string()).unwrap_or(Default::default()),
                    f.period_number,
                    f.protons_per_pulse,
                    f.running,
                );
            }
        }
    }
}

// Message dumping tool
pub(crate) async fn run(args: FrameOpts) -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut frames: HashMap<FrameNumber, (Vec<MessageData>,Vec<MessageData>)> = HashMap::new();

    let consumer: StreamConsumer = supermusr_common::generate_kafka_client_config(
        &args.common.broker,
        &args.common.username,
        &args.common.password,
    )
    .set("group.id", &args.common.consumer_group)
    .set("enable.partition.eof", "false")
    .set("session.timeout.ms", "6000")
    .set("enable.auto.commit", "false")
    .create()?;

    consumer.subscribe(&[&args.common.topic, &args.extra_topic])?;

    loop {
        match consumer.recv().await {
            Err(e) => warn!("Kafka error: {}", e),
            Ok(msg) => {
                debug!(
                    "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    msg.key(),
                    msg.topic(),
                    msg.partition(),
                    msg.offset(),
                    msg.timestamp()
                );

                if let Some(payload) = msg.payload() {
                    if digitizer_event_list_message_buffer_has_identifier(payload) {
                        match root_as_digitizer_event_list_message(payload) {
                            Ok(data) => {
                                {
                                    let entry = {
                                        if let Some(entry) =
                                            frames.get_mut(&data.metadata().frame_number())
                                        {
                                            entry
                                        } else {
                                            frames.insert(
                                                data.metadata().frame_number(),
                                                Default::default(),
                                            );
                                            frames.get_mut(&data.metadata().frame_number()).unwrap()
                                        }
                                    };
                                    entry.0.push(MessageData {
                                        id: data.digitizer_id(),
                                        timestamp: data
                                            .metadata()
                                            .timestamp()
                                            .and_then(|ts| (*ts).try_into().ok()),
                                        period_number: data.metadata().period_number(),
                                        protons_per_pulse: data.metadata().protons_per_pulse(),
                                        running: data.metadata().running(),
                                        veto_flags: data.metadata().veto_flags(),
                                    });
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse message: {}", e);
                            }
                        }
                    } else if digitizer_analog_trace_message_buffer_has_identifier(payload) {
                        match root_as_digitizer_analog_trace_message(payload) {
                            Ok(data) => {
                                {
                                    let entry = {
                                        if let Some(entry) =
                                            frames.get_mut(&data.metadata().frame_number())
                                        {
                                            entry
                                        } else {
                                            frames.insert(
                                                data.metadata().frame_number(),
                                                Default::default(),
                                            );
                                            frames.get_mut(&data.metadata().frame_number()).unwrap()
                                        }
                                    };
                                    entry.1.push(MessageData {
                                        id: data.digitizer_id(),
                                        timestamp: data
                                            .metadata()
                                            .timestamp()
                                            .and_then(|ts| (*ts).try_into().ok()),
                                        period_number: data.metadata().period_number(),
                                        protons_per_pulse: data.metadata().protons_per_pulse(),
                                        running: data.metadata().running(),
                                        veto_flags: data.metadata().veto_flags(),
                                    });
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse message: {}", e);
                            }
                        }
                    }
                    else {
                        warn!("Unexpected message type on topic \"{}\"", msg.topic());
                    }
                    if frames.len() >= args.frames_to_collect {
                        dump(&frames);
                        return Ok(());
                    }
                }

                consumer.commit_message(&msg, CommitMode::Async).unwrap();
            }
        };
    }
}
