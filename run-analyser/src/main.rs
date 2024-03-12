mod analysis;
mod message;

use clap::Parser;
use message::PairOfEventListByChannel;
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use std::{
    fmt::Debug, fs::File, net::SocketAddr, path::PathBuf
};
use supermusr_streaming_types::{
    aev1_frame_assembled_event_v1_generated::{
        frame_assembled_event_list_message_buffer_has_identifier,
        root_as_frame_assembled_event_list_message
    },
    ecs_6s4t_run_stop_generated::{
        root_as_run_stop,
        run_stop_buffer_has_identifier
    },
    ecs_pl72_run_start_generated::{
        root_as_run_start,
        run_start_buffer_has_identifier
    }
};
use tracing::{debug, info, warn};

use crate::analysis::ChannelPairAnalysis;

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
    control_topic: String,

    #[clap(long)]
    detected_events_topic: String,

    #[clap(long)]
    expected_frames: Option<usize>,

    #[clap(long)]
    simulated_events_topic: String,

    #[clap(long, env, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    #[clap(long)]
    path: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    tracing_subscriber::fmt::init();

    let mut client_config = supermusr_common::generate_kafka_client_config(
        &args.broker,
        &args.username,
        &args.password,
    );

    let consumer: StreamConsumer = client_config
        .set("group.id", &args.consumer_group)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()
        .expect("Kafka Consumer should be created");

    consumer
        .subscribe(&[&args.control_topic, &args.detected_events_topic, &args.simulated_events_topic])
        .expect("Kafka Consumer should subscribe to given topics");

    File::options()
        .truncate(true)
        .write(true)
        .create(true)
        .open(&args.path)
        .unwrap();

    let mut pair_of_eventlists_by_channel = PairOfEventListByChannel::new();

    let mut num_detected_frames = 0;
    let mut num_simulated_frames = 0;

    loop {
        match consumer.recv().await {
            Ok(m) => {
                debug!(
                    "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    m.key(),
                    m.topic(),
                    m.partition(),
                    m.offset(),
                    m.timestamp()
                );

                if let Some(payload) = m.payload() {
                    if m.topic() == &args.detected_events_topic || m.topic() == &args.simulated_events_topic {
                        if frame_assembled_event_list_message_buffer_has_identifier(payload) {
                            match root_as_frame_assembled_event_list_message(payload) {
                                Ok(thing) => {
                                    if m.topic() == &args.detected_events_topic {
                                        info!("New detected frame message");
                                        num_detected_frames += 1;
                                    } else {
                                        info!("New simulated frame message");
                                        num_simulated_frames += 1;
                                    }
                                    for c in thing.channel().unwrap().iter() {
                                        let event_list = {
                                            let pair = pair_of_eventlists_by_channel.entry(c).or_default();

                                            if m.topic() == &args.detected_events_topic {
                                                &mut pair.detected
                                            } else {
                                                &mut pair.simulated
                                            }
                                        };
                                        event_list.time.extend(thing.time().unwrap());
                                        event_list.voltage.extend(thing.voltage().unwrap());
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to parse message: {}", e);
                                }
                            }
                        } else {
                            warn!("Unexpected message type on event topic \"{}\"", m.topic());
                        }
                    } else if m.topic() == &args.control_topic {
                        if run_start_buffer_has_identifier(payload) {
                            match root_as_run_start(payload) {
                                Ok(_) => {
                                    info!("Run Start");
                                    pair_of_eventlists_by_channel.clear();
                                }
                                Err(e) => {
                                    warn!("Failed to parse message: {}", e);
                                }
                            }
                        } else if run_stop_buffer_has_identifier(payload) {
                            match root_as_run_stop(payload) {
                                Ok(_) => {
                                    info!("Run Stop");
                                    for (c,el) in pair_of_eventlists_by_channel.iter() {
                                        let analysis = ChannelPairAnalysis::analyse_channel(el);
                                        println!("Channel {c}:\n{analysis}");
                                    }
                                    consumer.commit_message(&m, CommitMode::Sync).unwrap();
                                    return;
                                }
                                Err(e) => {
                                    warn!("Failed to parse message: {}", e);
                                }
                            }
                        } else {
                            warn!("Unexpected message type on control topic ");
                        }
                    } else {
                        warn!("Unexpected topic: \"{}\"", m.topic());
                    }
                } else {
                    warn!("Unexpected message type on topic \"{}\"", m.topic());
                }

                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
            Err(e) => warn!("Kafka error: {}", e),
        };
        if let Some(expected_frames) = &args.expected_frames {
            if num_detected_frames == *expected_frames && num_simulated_frames == *expected_frames {
                info!("Expected Frames Found");
                for (c,el) in pair_of_eventlists_by_channel.iter() {
                    let analysis = ChannelPairAnalysis::analyse_channel(el);
                    println!("Channel {c}:\n{analysis}");
                }
                return;
            }
        }
    }
}
/*
fn write_analysis(path: &Path, analysis_key: &AnalysisKey, analysis: FramePairAnalysis) {
    let file = File::options()
        .append(true)
        .create(true)
        .open(path)
        .unwrap();

    writeln!(
        &file,
        "{0},{1}|{analysis}",
        analysis_key.digitiser_id, analysis_key.frame_number
    )
    .unwrap();
}
 */