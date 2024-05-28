mod data;
mod frame;

use crate::data::EventData;
use clap::Parser;
use frame::FrameCache;
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::{BorrowedMessage, Message},
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::{fmt::Debug, net::SocketAddr, time::Duration};
use supermusr_common::{
    conditional_init_tracer,
    spanned::Spanned,
    tracer::{
        FutureRecordTracerExt, OptionalHeaderTracerExt, OtelOptions, TracerEngine, TracerOptions,
    },
    DigitizerId,
};
use supermusr_streaming_types::{
    dev2_digitizer_event_v2_generated::{
        digitizer_event_list_message_buffer_has_identifier, root_as_digitizer_event_list_message,
    },
    FrameMetadata,
};
use tracing::{debug, error, info_span, level_filters::LevelFilter, warn};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(long)]
    broker: String,

    #[clap(long)]
    username: Option<String>,

    #[clap(long)]
    password: Option<String>,

    #[clap(long = "group")]
    consumer_group: String,

    #[clap(long)]
    input_topic: String,

    #[clap(long)]
    output_topic: String,

    #[clap(short, long, value_delimiter = ',')]
    digitiser_ids: Vec<DigitizerId>,

    #[clap(long, default_value = "500")]
    frame_ttl_ms: u64,

    #[clap(long, default_value = "500")]
    cache_poll_ms: u64,

    #[clap(long, env, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    /// If set, then open-telemetry data is sent to the URL specified, otherwise the standard tracing subscriber is used
    #[clap(long)]
    otel_endpoint: Option<String>,

    /// If open-telemetry is used then the following log level is used
    #[clap(long, default_value = "info")]
    otel_level: LevelFilter,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let tracer = conditional_init_tracer!(TracerOptions {
        otel_options: args.otel_endpoint.as_deref().map(|endpoint| OtelOptions {
            endpoint,
            level_filter: args.otel_level
        })
    });

    let consumer: StreamConsumer = supermusr_common::generate_kafka_client_config(
        &args.broker,
        &args.username,
        &args.password,
    )
    .set("group.id", &args.consumer_group)
    .set("enable.partition.eof", "false")
    .set("session.timeout.ms", "6000")
    .set("enable.auto.commit", "false")
    .create()
    .expect("kafka consumer should be created");

    consumer
        .subscribe(&[&args.input_topic])
        .expect("kafka topic should be subscribed");

    let producer = supermusr_common::generate_kafka_client_config(
        &args.broker,
        &args.username,
        &args.password,
    )
    .create()
    .expect("Kafka producer should be created");

    let ttl = Duration::from_millis(args.frame_ttl_ms);

    let mut cache = FrameCache::<EventData>::new(ttl, args.digitiser_ids.clone());

    let mut cache_poll_interval = tokio::time::interval(Duration::from_millis(args.cache_poll_ms));
    loop {
        tokio::select! {
            event = consumer.recv() => {
                match event {
                    Ok(msg) => {
                        on_message(tracer.is_some(), &mut cache, &producer, &args.output_topic, &msg).await;
                        consumer.commit_message(&msg, CommitMode::Async)
                            .unwrap();
                    }
                    Err(e) => warn!("Kafka error: {}", e),
                };
            }
            _ = cache_poll_interval.tick() => {
                cache_poll(tracer.is_some(), &mut cache, &producer, &args.output_topic).await;
            }
        }
    }
}

#[tracing::instrument(skip_all)]
async fn on_message(
    use_otel: bool,
    cache: &mut FrameCache<EventData>,
    producer: &FutureProducer,
    output_topic: &str,
    msg: &BorrowedMessage<'_>,
) {
    msg.headers().conditional_extract_to_current_span(use_otel);

    if let Some(payload) = msg.payload() {
        if digitizer_event_list_message_buffer_has_identifier(payload) {
            match root_as_digitizer_event_list_message(payload) {
                Ok(msg) => {
                    let metadata_result: Result<FrameMetadata, _> = msg.metadata().try_into();
                    match metadata_result {
                        Ok(metadata) => {
                            debug!("Event packet: metadata: {:?}", msg.metadata());
                            cache.push(msg.digitizer_id(), metadata.clone(), msg.into());

                            //let root_span = cache.get_root_span().clone();
                            if let Some(frame_span) = cache.find_span(metadata) {
                                if frame_span.is_waiting() {
                                    frame_span
                                        .init(info_span!(target: "otel", parent: None, "Frame", is_completed = tracing::field::Empty, is_expired = tracing::field::Empty, digitisers = tracing::field::Empty))
                                        .unwrap();
                                }
                                let cur_span = tracing::Span::current();
                                frame_span.get().unwrap().in_scope(|| {
                                    let span = info_span!(target: "otel", "Digitiser Event List", did = msg.digitizer_id());
                                    span.follows_from(cur_span);
                                });
                            }
                            cache_poll(use_otel, cache, producer, output_topic).await;
                        }
                        Err(e) => warn!("Invalid Metadata: {e}"),
                    }
                }
                Err(e) => {
                    warn!("Failed to parse message: {}", e);
                }
            }
        } else {
            warn!("Unexpected message type on topic \"{}\"", msg.topic());
            debug!("Message: {msg:?}");
            debug!("Payload size: {}", payload.len());
        }
    }
}

#[tracing::instrument(skip_all, level = "trace")]
async fn cache_poll(
    use_otel: bool,
    cache: &mut FrameCache<EventData>,
    producer: &FutureProducer,
    output_topic: &str,
) {
    if let Some(frame) = cache.poll() {
        let span = frame.span().get().unwrap().clone();
        let data: Vec<u8> = frame.into();

        let future_record = FutureRecord::to(output_topic)
            .payload(data.as_slice())
            .conditional_inject_span_into_headers(use_otel, &span)
            .key("Frame Events List");

        let pub_span = info_span!(target: "otel", parent: &span, "Publish Message");
        let _guard = pub_span.enter();
        match producer
            .send(future_record, Timeout::After(Duration::from_millis(100)))
            .await
        {
            Ok(r) => debug!("Delivery: {:?}", r),
            Err(e) => error!("Delivery failed: {:?}", e),
        };
    }
}
