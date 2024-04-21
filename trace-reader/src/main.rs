use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use rand::{seq::IteratorRandom, thread_rng};
use rdkafka::{
    message::OwnedHeaders,
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::{path::PathBuf, time::Duration};
use supermusr_common::{tracer::OtelTracer, DigitizerId, FrameNumber};
use supermusr_streaming_types::flatbuffers::FlatBufferBuilder;
use tracing::{debug, error, level_filters::LevelFilter, trace_span};

mod loader;
mod processing;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    /// Kafka message broker, should have format `host:port`, e.g. `localhost:19092`
    #[clap(long)]
    broker: String,

    /// Optional Kafka username
    #[clap(long)]
    username: Option<String>,

    /// Optional Kafka password
    #[clap(long)]
    password: Option<String>,

    /// Name of the Kafka consumer group
    #[clap(long)]
    consumer_group: String,

    /// The Kafka topic that trace messages are produced to
    #[clap(long)]
    trace_topic: String,

    /// Relative path to the .trace file to be read
    #[clap(long)]
    file_name: PathBuf,

    /// The frame number to assign the message
    #[clap(long, default_value = "0")]
    frame_number: FrameNumber,

    /// The digitizer id to assign the message
    #[clap(long, default_value = "0")]
    digitizer_id: DigitizerId,

    /// The number of trace events to bypass before reading (only effective when not using random sampling)
    #[clap(long, default_value = "0")]
    trace_offset: usize,

    /// The number of trace events to read. If zero, then all trace events are read
    #[clap(long, default_value = "1")]
    number_of_trace_events: usize,

    /// If set, then trace events are sampled randomly with replacement, if not set then trace events are read in order
    #[clap(long, default_value = "false")]
    random_sample: bool,

    /// If set, then open-telemetry data is sent to the URL specified, otherwise the standard tracing subscriber is used
    #[clap(long)]
    otel_endpoint: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let _tracer = init_tracer(args.otel_endpoint.as_ref());

    let span = trace_span!("TraceReader");
    let _guard = span.enter();

    let client_config = supermusr_common::generate_kafka_client_config(
        &args.broker,
        &args.username,
        &args.password,
    );

    let producer: FutureProducer = client_config
        .create()
        .expect("Kafka Producer should be created");

    run_reader(&args, &producer).await.expect("Reader is Run")
}

async fn run_reader(args: &Cli, producer: &FutureProducer) -> Result<()> {
    let mut trace_file =
        loader::load_trace_file(args.file_name.clone()).expect("Trace File should load");
    let total_trace_events = trace_file.get_number_of_trace_events();
    let num_trace_events = {
        if args.number_of_trace_events == 0 {
            total_trace_events
        } else {
            args.number_of_trace_events
        }
    };

    let trace_event_indices: Vec<_> = {
        if args.random_sample {
            (0..num_trace_events)
                .map(|_| {
                    (0..num_trace_events)
                        .choose(&mut thread_rng())
                        .unwrap_or_default()
                })
                .collect()
        } else {
            (0..num_trace_events)
                .cycle()
                .skip(args.trace_offset)
                .take(num_trace_events)
                .collect()
        }
    };

    let mut fbb = FlatBufferBuilder::new();
    for index in trace_event_indices {
        let span = trace_span!("ReadTraceEvent");
        let _guard = span.enter();

        let event = trace_file.get_trace_event(index)?;
        processing::create_message(
            &mut fbb,
            Utc::now().into(),
            args.frame_number,
            args.digitizer_id,
            trace_file.get_num_channels(),
            (1.0 / trace_file.get_sample_time()) as u64,
            &event,
        )?;

        let future_record = {
            if args.otel_endpoint.is_some() {
                let mut headers = OwnedHeaders::new();
                OtelTracer::inject_context_from_span_into_kafka(&span, &mut headers);

                FutureRecord::to(&args.trace_topic)
                    .payload(fbb.finished_data())
                    .headers(headers)
                    .key("Trace")
            } else {
                FutureRecord::to(&args.trace_topic)
                    .payload(fbb.finished_data())
                    .key("Trace")
            }
        };

        let timeout = Timeout::After(Duration::from_millis(6000));
        match producer.send(future_record, timeout).await {
            Ok(r) => debug!("Delivery: {:?}", r),
            Err(e) => error!("Delivery failed: {:?}", e.0),
        };
    }
    Ok(())
}

fn init_tracer(otel_endpoint: Option<&String>) -> Option<OtelTracer> {
    otel_endpoint
        .map(|otel_endpoint| {
            OtelTracer::new(
                otel_endpoint,
                "Trace Reader",
                Some(("trace_reader", LevelFilter::TRACE)),
            )
            .expect("Open Telemetry Tracer is created")
        })
        .or_else(|| {
            tracing_subscriber::fmt::init();
            None
        })
}
