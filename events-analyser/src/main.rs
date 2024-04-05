mod analysis;
mod base;
mod message_group;
mod message_pair;

use analysis::FramePairAnalysis;
use clap::Parser;
use message_group::{ChannelEventList, MessageExtractable, MessageGroupContainer};
use message_pair::{MessagePair, MessagePairVectorContainer};
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use supermusr_common::tracer::{OtelTracer, Spanned};
use std::{
    fmt::Debug,
    fs::File,
    io::Write,
    net::SocketAddr,
    path::{Path, PathBuf},
};
use supermusr_streaming_types::dev1_digitizer_event_v1_generated::{
    digitizer_event_list_message_buffer_has_identifier, root_as_digitizer_event_list_message,
};
use tracing::{debug, info, trace_span, warn};

use analysis::analyse;
use base::{AnalysisKey, MessageKey};
use message_group::{DetectedMessage, Header};

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
    trace_to_events_topic: String,

    #[clap(long)]
    simulated_events_topic: Option<String>,

    #[clap(long)]
    expected_repetitions: usize,

    #[clap(long, env, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    #[clap(long)]
    path: PathBuf,
}

#[tokio::main]
async fn main() {
    let _tracer = OtelTracer::new().expect("Tracer should be created");

    let args = Cli::parse();

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

    let topics : Vec<&str> = if let Some(sim_events) = args.simulated_events_topic.as_ref() {
        vec![&args.trace_to_events_topic, sim_events.as_str()]
    } else {
        vec![&args.trace_to_events_topic]
    };
    consumer
        .subscribe(&topics)
        .expect("Kafka Consumer should subscribe to given topics");

    File::options()
        .truncate(true)
        .write(true)
        .create(true)
        .open(&args.path)
        .unwrap();

    let mut message_groups = MessageGroupContainer::new();
    let mut message_pair_vectors = MessagePairVectorContainer::new();

    loop {
        match consumer.recv().await {
            Ok(m) => {
                let span = trace_span!("analysis");
                OtelTracer::extract_context_from_kafka_to_span(m.headers().unwrap(), &span);
                let _guard = span.enter();
                //let context = m.headers().map(|headers|extract_context(headers));

                //let context = create_new_span("analysis", Some(context.unwrap()));
                //let _guard = context.attach();
                debug!(
                    "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    m.key(),
                    m.topic(),
                    m.partition(),
                    m.offset(),
                    m.timestamp()
                );

                if let Some(payload) = m.payload() {
                    if digitizer_event_list_message_buffer_has_identifier(payload) {
                        match root_as_digitizer_event_list_message(payload) {
                            Ok(thing) => {
                                let key = MessageKey::new(&thing);
                                let message_group = message_groups.entry(key.clone()).or_default();

                                if let Some(sim_event) = args.simulated_events_topic.as_ref() {
                                    if m.topic() == args.trace_to_events_topic {
                                        info!("Detected Events List  : {key:?}");
                                        message_group.detected = Some(Spanned {
                                            span: tracing::Span::current(),
                                            value: DetectedMessage {
                                                header: Header::from_message(&m),
                                                message: ChannelEventList::from_message(&thing),
                                            }
                                        });
                                    } else if m.topic() == sim_event {
                                        info!("Simulated Events List : {key:?}");
                                        message_group.simulated =
                                            Some(Spanned {
                                                span: tracing::Span::current(),
                                                value: ChannelEventList::from_message(&thing)
                                            }
                                        );
                                    }
                                } else {
                                    let channel_event_list = ChannelEventList::from_message(&thing);
                                    info!("Events List           : {key:?}");
                                    message_group.detected = Some(Spanned {
                                        span: tracing::Span::current(),
                                        value: DetectedMessage {
                                            header: Header::from_message(&m),
                                            message: channel_event_list.clone(),
                                        }
                                    });
                                    message_group.simulated = Some(Spanned {
                                        span: tracing::Span::current(),
                                        value: channel_event_list
                                    });
                                }

                                if let Some(spanned_pair) = MessagePair::from_message_group(message_group) {
                                    info!("Pair Matched");
                                    message_groups.remove(&key);
                                    let vec = message_pair_vectors
                                        .entry(key.analysis_key.clone())
                                        .or_default();

                                    vec.push(spanned_pair.value);
                                    if vec.len() == args.expected_repetitions {
                                        info!("Analysis written      : {0:?}",key.analysis_key);
                                        write_analysis(&args.path, &key.analysis_key, analyse(vec));
                                        message_pair_vectors.remove(&key.analysis_key);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse message: {}", e);
                            }
                        }
                    } else {
                        warn!("Unexpected message type on topic \"{}\"", m.topic());
                    }
                }

                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
            Err(e) => warn!("Kafka error: {}", e),
        };
    }
}

#[tracing::instrument]
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
