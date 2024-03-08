use chrono::{DateTime, Utc};
use clap::Parser;
use rand::{seq::IteratorRandom, thread_rng};
use rdkafka::producer::FutureProducer;
use std::path::PathBuf;
use supermusr_common::{Channel, DigitizerId, FrameNumber};

mod loader;
mod processing;
use loader::load_trace_file;
use processing::dispatch_trace_file;

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

    /// The Kafka topic that trace messages are produced to
    #[clap(long)]
    trace_topic: String,

    /// Relative path to the .trace file to be read
    #[clap(long)]
    path: PathBuf,

    /// Timestamp of the command, defaults to now, if not given.
    #[clap(long)]
    time: Option<DateTime<Utc>>,

    /// The frame number to assign the message
    #[clap(long, default_value = "0")]
    frame_number: FrameNumber,

    /// The digitizer id to assign the message
    #[clap(long, default_value = "0")]
    digitizer_id: DigitizerId,

    /// The number of trace events to read. If zero, then all trace events are read
    #[clap(long, default_value = "1")]
    number_of_trace_events: usize,

    /// Add this value to the channel ids
    #[clap(long, default_value = "0")]
    channel_id_offset: Channel,

    /// The amount of time to add between each frame
    #[clap(long, default_value = "0")]
    frame_interval_ms: i32,

    /// If set, then trace events are sampled randomly with replacement, if not set then trace events are read in order
    #[clap(long, default_value = "false")]
    random_sample: bool,

    /// Every channel index is shifted by this amount
    #[clap(long, default_value = "0")]
    channel_index_shift: Channel,

    /// The number of times to repeat each frame
    #[clap(long, default_value = "1")]
    repeat: usize,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Cli::parse();

    let client_config = supermusr_common::generate_kafka_client_config(
        &args.broker,
        &args.username,
        &args.password,
    );

    let producer: FutureProducer = client_config
        //.set("max.request.size","100000000")
        .create()
        .expect("Kafka Producer should be created");

    let trace_file = load_trace_file(args.path).expect("Trace File should load");
    let total_trace_events = trace_file.get_number_of_trace_events();
    let num_trace_events = if args.number_of_trace_events == 0 {
        total_trace_events
    } else {
        args.number_of_trace_events
    }*args.repeat;

    let trace_event_indices : Vec<_> = if args.random_sample {
        (0..num_trace_events)
            .map(|_| {
                (0..total_trace_events)
                    .choose(&mut thread_rng())
                    .unwrap_or_default()
            })
            .collect()
    } else {
        (0..total_trace_events)
            .cycle()
            .take(num_trace_events)
            .collect()
    };

    let trace_event_indices: Vec<_> = trace_event_indices.chunks(args.repeat).collect();

    let time = args.time.unwrap_or(Utc::now());
    dispatch_trace_file(
        trace_file,
        trace_event_indices,
        time,
        args.frame_number,
        args.digitizer_id,
        &producer,
        &args.trace_topic,
        6000,
        args.channel_id_offset,
        args.frame_interval_ms
    )
    .await
    .expect("Trace File should be dispatched to Kafka");
}
