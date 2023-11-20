//! This crate uses the benchmarking tool for testing the performance of implementated time-series databases.
//!
//#![allow(dead_code, unused_variables, unused_imports)]
#![warn(missing_docs)]

use clap::Parser;

use log::{debug, info, warn};

mod tdengine;
use tdengine as engine;

use anyhow::Result;

use engine::{tdengine::TDEngine, TimeSeriesEngine};

use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};

use streaming_types::dat1_digitizer_analog_trace_v1_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
};

#[cfg(feature = "benchmark")]
mod benchmark;

#[cfg(feature = "benchmark")]
use benchmark::BenchmarkData;

//mod full_test;

//cargo run -- --kafka-broker=localhost:19092 --kafka-topic=Traces --td-broker=172.16.105.238:6041 --td-database=tracelogs --td-num-channels=8

#[derive(Parser)]
#[clap(author, version, about)]
pub(crate) struct Cli {
    #[clap(long, short = 'b', env = "KAFKA_BROKER")]
    kafka_broker: String,

    #[clap(long, short = 'u', env = "KAFKA_USER")]
    kafka_username: Option<String>,

    #[clap(long, short = 'p', env = "KAFKA_PASSWORD")]
    kafka_password: Option<String>,

    #[clap(
        long,
        short = 'g',
        env = "KAFKA_CONSUMER_GROUP",
        default_value = "trace-consumer"
    )]
    kafka_consumer_group: String,

    #[clap(long, short = 'k', env = "KAFKA_TOPIC")]
    kafka_topic: String,

    #[clap(long, short = 'B', env = "TDENGINE_BROKER")]
    td_broker: String,

    #[clap(long, short = 'U', env = "TDENGINE_USER")]
    td_username: Option<String>,

    #[clap(long, short = 'P', env = "TDENGINE_PASSWORD")]
    td_password: Option<String>,

    #[clap(long, short = 'D', env = "TDENGINE_DATABASE")]
    td_database: String,

    #[clap(long, short = 'C', env = "TDENGINE_NUM_CHANNELS")]
    td_num_channels: usize,

    #[clap(long, short = 's', env = "TDENGINE_BATCH_SIZE", default_value = "1")]
    batch_size: usize,

    #[cfg(feature = "benchmark")]
    #[clap(
        short = 'n',
        long,
        help = "If set, will record benchmarking data",
        default_value = "0"
    )]
    benchmark_number: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    debug!("Parsing Cli");
    let cli = Cli::parse();

    //  All other modes require a TDEngine instance
    debug!("Createing TDEngine instance");
    let mut tdengine: TDEngine = TDEngine::new(
        cli.td_broker,
        cli.td_username,
        cli.td_password,
        cli.td_database,
        cli.td_num_channels,
        cli.batch_size,
    )
    .await?;

    //  All other modes require the TDEngine to be initialised
    tdengine.create_database().await?;
    tdengine
        .init()
        .await?;

    //  All other modes require a kafka builder, a topic, and redpanda consumer
    debug!("Creating Kafka instance");

    let mut client_config = common::generate_kafka_client_config(
        &cli.kafka_broker,
        &cli.kafka_username,
        &cli.kafka_password,
    );

    let consumer: StreamConsumer = client_config
        .set("group.id", &cli.kafka_consumer_group)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()?;
    consumer.subscribe(&[&cli.kafka_topic])?;

    #[cfg(feature = "benchmark")]
    let mut benchmark_data = BenchmarkData::new(cli.benchmark_number);
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

                                    #[cfg(feature = "benchmark")]
                                    benchmark_data.begin_processing_timer();

                                    if let Err(e) = tdengine.process_message(&message).await {
                                        warn!("Error processing message : {e}");
                                    }

                                    #[cfg(feature = "benchmark")]
                                    benchmark_data.begin_posting_timer();

                                    if let Err(e) = tdengine.post_message().await {
                                        warn!("Error posting message to tdengine : {e}");
                                    }
                                    #[cfg(feature = "benchmark")]
                                    benchmark_data.end_timers();
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

        #[cfg(feature = "benchmark")]
        benchmark_data.print_times();
    }
}
