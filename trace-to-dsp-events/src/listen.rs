use anyhow::Result;

use common::{Channel, EventData, Intensity, Time};
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
    producer::{FutureProducer, FutureRecord},
};

use std::{net::SocketAddr, time::Duration};
use streaming_types::{
    dat1_digitizer_analog_trace_v1_generated::{
        digitizer_analog_trace_message_buffer_has_identifier,
        root_as_digitizer_analog_trace_message, ChannelTrace, DigitizerAnalogTraceMessage,
    },
    dev1_digitizer_event_v1_generated::{
        finish_digitizer_event_list_message_buffer, DigitizerEventListMessage,
        DigitizerEventListMessageArgs,
    },
    flatbuffers::FlatBufferBuilder,
    frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args},
};

use crate::trace_run;

struct ChannelEvents {
    channel_number: Channel,

    time: Vec<Time>,
    voltage: Vec<Intensity>,
}

pub(super) async fn listen(
    consumer: &StreamConsumer,
    producer: &FutureProducer,
    event_topic: &str,
) -> Result<()> {
    match consumer.recv().await {
        Ok(m) => {
            log::debug!(
                "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                m.key(),
                m.topic(),
                m.partition(),
                m.offset(),
                m.timestamp()
            );

            if let Some(payload) = m.payload() {
                if digitizer_analog_trace_message_buffer_has_identifier(payload) {
                    match root_as_digitizer_analog_trace_message(payload) {
                        Ok(thing) => {
                            let bytes = process_message(&thing);
                            let key = m.offset().to_string();
                            let future_record =
                                FutureRecord::to(event_topic).payload(&bytes).key(&key);
                            match producer.send(future_record, Duration::from_secs(0)).await {
                                Ok(_) => log::trace!("Published event message"),
                                Err(e) => log::error!("{:?}", e),
                            }
                        }
                        Err(e) => log::warn!("Failed to parse message: {}", e),
                    }
                } else {
                    log::warn!("Unexpected message type on topic \"{}\"", m.topic());
                }
            }

            consumer.commit_message(&m, CommitMode::Sync).unwrap();
        }
        Err(e) => log::warn!("Kafka error: {}", e),
    };
    Ok(())
}

pub(crate) fn process_message(message: &DigitizerAnalogTraceMessage) -> Vec<u8> {
    log::info!(
        "Dig ID: {}, Metadata: {:?}",
        message.digitizer_id(),
        message.metadata()
    );

    let mut fbb = FlatBufferBuilder::new();

    let mut events = common::EventData::default();

    /*let sample_time_in_us: Time = (1_000_000 / message.sample_rate())
    .try_into()
    .expect("Sample time range");*/

    let channel_traces: Vec<Vec<_>> = message
        .channels()
        .unwrap()
        .iter()
        .map(|i| i.voltage().unwrap().into_iter().collect())
        .collect();

    let channel_events: Vec<ChannelEvents> = channel_traces
        .iter()
        .map(|trace| trace_run::run_detection(&trace))
        .enumerate()
        .map(|(c, vec_pulse)| ChannelEvents {
            channel_number: c as Channel,
            time: vec_pulse
                .iter()
                .map(|pulse| pulse.steepest_rise.time.unwrap_or(-1.0) as Time)
                .collect(),
            voltage: vec_pulse
                .iter()
                .map(|pulse| pulse.peak.value.unwrap_or_default() as Intensity)
                .collect(),
        })
        .collect();

    for mut channel in channel_events {
        events
            .channel
            .append(&mut vec![channel.channel_number; channel.time.len()]);
        events.time.append(&mut channel.time);
        events.voltage.append(&mut channel.voltage);
    }

    let metadata = FrameMetadataV1Args {
        frame_number: message.metadata().frame_number(),
        period_number: message.metadata().period_number(),
        running: message.metadata().running(),
        protons_per_pulse: message.metadata().protons_per_pulse(),
        timestamp: message.metadata().timestamp(),
        veto_flags: message.metadata().veto_flags(),
    };
    let metadata = FrameMetadataV1::create(&mut fbb, &metadata);

    let time = Some(fbb.create_vector(&events.time));
    let voltage = Some(fbb.create_vector(&events.voltage));
    let channel = Some(fbb.create_vector(&events.channel));

    let message = DigitizerEventListMessageArgs {
        digitizer_id: message.digitizer_id(),
        metadata: Some(metadata),
        time,
        voltage,
        channel,
    };
    let message = DigitizerEventListMessage::create(&mut fbb, &message);
    finish_digitizer_event_list_message_buffer(&mut fbb, message);

    fbb.finished_data().to_vec()
}
