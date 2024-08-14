mod data;
mod frame;

use crate::data::EventData;
use clap::Parser;
use frame::FrameCache;
use metrics::counter;
use metrics_exporter_prometheus::PrometheusBuilder;
use rdkafka::{
    consumer::{CommitMode, Consumer},
    message::{BorrowedHeaders, BorrowedMessage, Message},
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::{fmt::Debug, net::SocketAddr, time::Duration};
use supermusr_common::{
    init_tracer,
    metrics::{
        messages_received::{self, MessageKind},
        metric_names::{FAILURES, FRAMES_SENT, MESSAGES_PROCESSED, MESSAGES_RECEIVED},
    },
    spanned::{FindSpanMut, Spanned},
    tracer::{FutureRecordTracerExt, OptionalHeaderTracerExt, TracerEngine, TracerOptions},
    CommonKafkaOpts, DigitizerId,
};
use supermusr_streaming_types::{
    dev2_digitizer_event_v2_generated::{
        digitizer_event_list_message_buffer_has_identifier, root_as_digitizer_event_list_message,
        DigitizerEventListMessage,
    },
    flatbuffers::InvalidFlatbuffer,
    FrameMetadata,
};
use tokio::task::JoinSet;
use tracing::{debug, error, info_span, instrument, level_filters::LevelFilter, warn, Instrument};

const TIMESTAMP_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.f%z";

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(flatten)]
    common_kafka_options: CommonKafkaOpts,

    /// Kafka consumer group
    #[clap(long = "group")]
    consumer_group: String,

    /// Kafka topic on which to listen for per digitiser event messages
    #[clap(long)]
    input_topic: String,

    /// Kafka topic on which to emit frame assembled event messages
    #[clap(long)]
    output_topic: String,

    /// A list of expected digitiser IDs.
    /// A frame is only "complete" when a message has been received from each of these IDs.
    #[clap(short, long)]
    digitiser_ids: Vec<DigitizerId>,

    /// Frame TTL in milliseconds.
    /// The time in which messages for a given frame must have been received from all digitisers.
    #[clap(long, default_value = "500")]
    frame_ttl_ms: u64,

    /// Frame cache poll interval in milliseconds.
    /// This may affect the rate at which incomplete frames are transmitted.
    #[clap(long, default_value = "500")]
    cache_poll_ms: u64,

    /// Endpoint on which Prometheus text format metrics are available
    #[clap(long, env, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    /// If set, then OpenTelemetry data is sent to the URL specified, otherwise the standard tracing subscriber is used
    #[clap(long)]
    otel_endpoint: Option<String>,

    /// The reporting level to use for OpenTelemetry
    #[clap(long, default_value = "info")]
    otel_level: LevelFilter,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let tracer = init_tracer!(TracerOptions::new(
        args.otel_endpoint.as_deref(),
        args.otel_level
    ));

    let kafka_opts = args.common_kafka_options;

    let consumer = supermusr_common::create_default_consumer(
        &kafka_opts.broker,
        &kafka_opts.username,
        &kafka_opts.password,
        &args.consumer_group,
        &[args.input_topic.as_str()],
    );

    let producer = supermusr_common::generate_kafka_client_config(
        &kafka_opts.broker,
        &kafka_opts.username,
        &kafka_opts.password,
    )
    .create()
    .expect("Kafka producer should be created");

    let ttl = Duration::from_millis(args.frame_ttl_ms);

    let mut cache = FrameCache::<EventData>::new(ttl, args.digitiser_ids.clone());

    // Install exporter and register metrics
    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(args.observability_address)
        .install()
        .expect("Prometheus metrics exporter should be setup");

    metrics::describe_counter!(
        MESSAGES_RECEIVED,
        metrics::Unit::Count,
        "Number of messages received"
    );
    metrics::describe_counter!(
        MESSAGES_PROCESSED,
        metrics::Unit::Count,
        "Number of messages processed"
    );
    metrics::describe_counter!(
        FAILURES,
        metrics::Unit::Count,
        "Number of failures encountered"
    );
    metrics::describe_counter!(
        FRAMES_SENT,
        metrics::Unit::Count,
        "Number of complete frames sent by the aggregator"
    );

    let mut kafka_producer_thread_set = JoinSet::new();

    let mut cache_poll_interval = tokio::time::interval(Duration::from_millis(args.cache_poll_ms));
    loop {
        tokio::select! {
            event = consumer.recv() => {
                match event {
                    Ok(msg) => {
                        process_kafka_message(tracer.use_otel(), &mut kafka_producer_thread_set, &mut cache, &producer, &args.output_topic, &msg).await;
                        consumer.commit_message(&msg, CommitMode::Async)
                            .unwrap();
                    }
                    Err(e) => warn!("Kafka error: {}", e),
                };
            }
            _ = cache_poll_interval.tick() => {
                cache_poll(tracer.use_otel(), &mut kafka_producer_thread_set, &mut cache, &producer, &args.output_topic).await;
            }
        }
    }
}

#[instrument(skip_all, target = "otel")]
fn spanned_root_as_digitizer_event_list_message(
    payload: &[u8],
) -> Result<DigitizerEventListMessage<'_>, InvalidFlatbuffer> {
    root_as_digitizer_event_list_message(payload)
}

#[instrument(skip_all, level = "info", fields(kafka_message_timestamp_ms = msg.timestamp().to_millis()))]
async fn process_kafka_message(
    use_otel: bool,
    kafka_producer_thread_set: &mut JoinSet<()>,
    cache: &mut FrameCache<EventData>,
    producer: &FutureProducer,
    output_topic: &str,
    msg: &BorrowedMessage<'_>,
) {
    if let Some(payload) = msg.payload() {
        if digitizer_event_list_message_buffer_has_identifier(payload) {
            counter!(
                MESSAGES_RECEIVED,
                &[messages_received::get_label(MessageKind::Event)]
            );
            let headers = msg.headers();
            match spanned_root_as_digitizer_event_list_message(payload) {
                Ok(msg) => {
                    process_digitiser_event_list_message(
                        use_otel,
                        headers,
                        kafka_producer_thread_set,
                        cache,
                        producer,
                        output_topic,
                        msg,
                    )
                    .await;
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

#[tracing::instrument(skip_all, fields(
    num_cached_frames = cache.get_num_partial_frames(),
    metadata_timestamp = tracing::field::Empty,
    metadata_frame_number = tracing::field::Empty,
    metadata_period_number = tracing::field::Empty,
    metadata_veto_flags = tracing::field::Empty,
    metadata_protons_per_pulse = tracing::field::Empty,
    metadata_running = tracing::field::Empty,
))]
async fn process_digitiser_event_list_message(
    use_otel: bool,
    headers: Option<&BorrowedHeaders>,
    kafka_producer_thread_set: &mut JoinSet<()>,
    cache: &mut FrameCache<EventData>,
    producer: &FutureProducer,
    output_topic: &str,
    msg: DigitizerEventListMessage<'_>,
) {
    let metadata_result: Result<FrameMetadata, _> = msg.metadata().try_into();
    match metadata_result {
        Ok(metadata) => {
            headers.conditional_extract_to_current_span(use_otel);
            {
                let span = tracing::Span::current();
                span.record("metadata_timestamp", metadata.timestamp.format(TIMESTAMP_FORMAT).to_string());
                span.record("metadata_frame_number", metadata.frame_number);
                span.record("metadata_period_number", metadata.period_number);
                span.record("metadata_veto_flags", metadata.veto_flags);
                span.record("metadata_protons_per_pulse", metadata.protons_per_pulse);
                span.record("metadata_running", metadata.running);
            }

            //debug!("Event packet: metadata: {:?}", msg.metadata());
            cache.push(msg.digitizer_id(), &metadata, msg.into());

            if let Some(frame_span) = cache.find_span_mut(&metadata) {
                if frame_span.is_waiting() {
                    frame_span
                        .init(info_span!(target: "otel", parent: None, "Frame",
                            "metadata_timestamp" = metadata.timestamp.format(TIMESTAMP_FORMAT).to_string(),
                            "metadata_frame_number" = metadata.frame_number,
                            "metadata_period_number" = metadata.period_number,
                            "metadata_veto_flags" = metadata.veto_flags,
                            "metadata_protons_per_pulse" = metadata.protons_per_pulse,
                            "metadata_running" = metadata.running,
                            "frame_is_complete" = tracing::field::Empty,
                            "frame_is_expired" = tracing::field::Empty,
                        ))
                        .unwrap();
                }
                let cur_span = tracing::Span::current();
                frame_span.get().unwrap().in_scope(|| {
                    info_span!(target: "otel", "Digitiser Event List").follows_from(cur_span);
                });
            }
            cache_poll(
                use_otel,
                kafka_producer_thread_set,
                cache,
                producer,
                output_topic,
            )
            .await;
        }
        Err(e) => warn!("Invalid Metadata: {e}"),
    }
}

#[tracing::instrument(skip_all, level = "trace")]
async fn cache_poll(
    use_otel: bool,
    kafka_producer_thread_set: &mut JoinSet<()>,
    cache: &mut FrameCache<EventData>,
    producer: &FutureProducer,
    output_topic: &str,
) {
    if let Some(frame) = cache.poll() {
        let span = info_span!(target: "otel", "Frame Complete");
        let _guard = span.enter();

        let frame_span = frame.span().get().unwrap().clone();
        let data: Vec<u8> = frame.into();

        let producer = producer.to_owned();
        let output_topic = output_topic.to_owned();
        let future = async move {
            let future_record = FutureRecord::to(&output_topic)
                .payload(data.as_slice())
                .conditional_inject_span_into_headers(use_otel, &frame_span)
                .key("Frame Events List");

            match producer
                .send(future_record, Timeout::After(Duration::from_millis(100)))
                .await
            {
                Ok(r) => debug!("Delivery: {:?}", r),
                Err(e) => error!("Delivery failed: {:?}", e),
            }
        };
        let span_clone = span.clone();
        kafka_producer_thread_set.spawn(
            future.instrument(info_span!(target: "otel", parent: span_clone, "Message Producer")),
        );
    }
}
