//! This crate uses the benchmarking tool for testing the performance of implementated time-series databases.
//!

mod database;
mod benchmark;

use clap::Parser;
use log::{debug, info, warn};
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use supermusr_streaming_types::dat1_digitizer_analog_trace_v1_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
};
use database::{tdengine::TDEngine, TimeSeriesEngine};

//RUST_LOG=warn cargo run --release -- --kafka-broker=localhost:19092 --kafka-topic=Traces --kafka-consumer-group trace-consumer --td-dsn 172.16.105.238:6030 --td-database=tracelogs --num-channels=8 -n 50 -s1

#[derive(Parser)]
#[clap(author, version, about)]
pub(crate) struct Cli {
    /// The kafka broker to use e.g. --broker localhost:19092
    #[clap(long)]
    kafka_broker: String,

    /// Optional Kafka username
    #[clap(long)]
    kafka_username: Option<String>,

    /// Optional Kafka password
    #[clap(long)]
    kafka_password: Option<String>,

    /// Kafka consumer group e.g. --kafka_consumer_group trace-producer
    #[clap(long)]
    kafka_consumer_group: String,

    /// Kafka topic e.g. --kafka-topic Traces
    #[clap(long)]
    kafka_topic: String,

    /// TDengine dsn  e.g. --td_dsn localhost:6041
    #[clap(long)]
    td_dsn: String,

    /// Optional TDengine username
    #[clap(long)]
    td_username: Option<String>,

    /// Optional TDengine password
    #[clap(long)]
    td_password: Option<String>,

    /// TDengine database name e.g. --td_database tracelogs
    #[clap(long)]
    td_database: String,

    /// Number of expected channels in a message e.g. --num_channels 8
    #[clap(long)]
    num_channels: usize,

    /// Number of expected channels in a message e.g. --num_channels 8
    #[clap(long, short = 's', default_value = "1")]
    batch_size: usize,

    /// Number of expected channels in a message e.g. --num_channels 8
    #[clap(long, short = 'n', default_value = "0")]
    num_tests: usize,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let cli = Cli::parse();

    debug!("Createing TDEngine instance");
    let mut tdengine: TDEngine = TDEngine::new(
        cli.td_dsn,
        cli.td_username,
        cli.td_password,
        cli.td_database,
        cli.num_channels,
        cli.batch_size,
        false

    )
    .await
    .expect("TDengine should be created");

    //  All other modes require the TDEngine to be initialised
    tdengine
        .create_database()
        .await
        .expect("TDengine database should be created");
    tdengine
        .init()
        .await
        .expect("TDengine should initialise with given channel count");

    //  All other modes require a kafka builder, a topic, and redpanda consumer
    debug!("Creating Kafka instance");
    let mut client_config = supermusr_common::generate_kafka_client_config(
        &cli.kafka_broker,
        &cli.kafka_username,
        &cli.kafka_password,
    );

    let consumer: StreamConsumer = client_config
        .set("group.id", &cli.kafka_consumer_group)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()
        .expect("Kafka Consumer should be created");
    consumer
        .subscribe(&[&cli.kafka_topic])
        .expect("Kafka Consumer should subscribe to kafka-topic");

    let mut benchmarker = benchmark::BenchmarkData::new(cli.num_tests, cli.batch_size);

    debug!("Begin Listening For Messages");
    loop {
        match consumer.recv().await {
            Ok(message) => {
                match message.payload() {
                    Some(payload) => {
                        if digitizer_analog_trace_message_buffer_has_identifier(payload) {
                            match root_as_digitizer_analog_trace_message(payload) {
                                Ok(message) => {
                                    info!(
                                        "Trace packet: dig. ID: {}, metadata: {:?}",
                                        message.digitizer_id(),
                                        message.metadata()
                                    );
                                    benchmarker.begin_processing_timer();
                                    if let Err(e) = tdengine.process_message(&message).await {
                                        warn!("Error processing message : {e}");
                                    }
                                    benchmarker.end_processing_timer();

                                    benchmarker.begin_binding_timer();
                                    if let Err(e) = tdengine.bind_samples().await {
                                        warn!("Error binding message to tdengine : {e}");
                                    }
                                    benchmarker.end_binding_timer();

                                    benchmarker.begin_posting_timer();
                                    if let Err(e) = tdengine.post_message().await {
                                        warn!("Error posting message to tdengine : {e}");
                                    }
                                    benchmarker.end_posting_timer();
                                    benchmarker.end_timers(message.channels().unwrap().get(0).voltage().unwrap().len());
                                }
                                Err(e) => warn!("Failed to parse message: {0}", e),
                            }
                        } else {
                            warn!("Message payload missing identifier.")
                        }
                    }
                    None => warn!("Error extracting payload from message."),
                };
                consumer
                    .commit_message(&message, CommitMode::Async)
                    .unwrap();
            }
            Err(e) => warn!("Error recieving message from server: {e}"),
        }
        benchmarker.print_times();
    }
}
