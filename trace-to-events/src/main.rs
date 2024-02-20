mod timer;
mod metrics;
mod parameters;
mod processing;
mod pulse_detection;

use clap::Parser;
use kagiyama::{AlwaysReady, Watcher};
use parameters::Mode;
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer}, message::{Header, Message}, producer::{FutureProducer, FutureRecord, Producer}, util::Timeout
};
use timer::StatTimer;
use std::{net::SocketAddr, path::PathBuf, time::Duration};
use supermusr_streaming_types::{dat1_digitizer_analog_trace_v1_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
}, flatbuffers::FlatBufferBuilder};
use tracing::{event, span, Instrument, Level};
use tracing_subscriber::{fmt, fmt::time};

use crate::timer::Data;
// cargo run --release --bin trace-to-events -- --broker localhost:19092 --trace-topic Traces --event-topic Events --group trace-to-events constant-phase-discriminator --threshold-trigger=-40,1,0
// cargo run --release --bin trace-to-events -- --broker localhost:19092 --trace-topic Traces --event-topic Events --group trace-to-events advanced-muon-detector --muon-onset=0.1 --muon-fall=0.1 --muon-termination=0.1 --duration=1
// RUST_LOG=off cargo run --release --bin trace-to-events -- --broker localhost:19092 --trace-topic Traces --event-topic Events --group trace-to-events advanced-muon-detector --muon-onset=0.1 --muon-fall=0.1 --muon-termination=0.1 --duration=1

// cargo run --release --bin trace-reader -- --broker localhost:19092 --consumer-group trace-producer --trace-topic Traces --file-name ../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces --number-of-trace-events 500 --channel-multiplier 4 --message-multiplier 1

/*
RUST_LOG=off cargo run --release --bin simulator -- --broker localhost:19092 --trace-topic Traces --num-channels 16 --time-bins 30000 continuous --frame-time 1
*/

/* Optimizations:
    Moving the fbb object out of the processing function and taking the slice rather than copying
    Streamline the process for writing channel event data to the message channel list
    Scoped multithreading to process channels simultaneously
    Change kafka property linger.ms to 0 (why does this help?)
    ^^^ Implementing async message producing with linger.ms at 100 or other
    Dispensed with pulse assembler in the case of constant phase discriminator (no apparent affect)

    Fixes:
    sampletime doesn't do anything in find_channel_events
    trace-reader: line 83 num_trace_events to total_trace_events

    Possible Optimizations:
    Employ multithreading for message passing.
*/
/*
            |  Constant  | Advanced
16 Channels | 1.5ms(0.5) | 12ms(3.0)
 8 Channels | 2.3ms(0.4) | 6ms (2.1)
 4 Channels | 1.2ms(0.2) | 3ms (1.6)
 stddev, min, max
 compression
 GPU
*/

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
    trace_topic: String,

    #[clap(long)]
    event_topic: String,

    #[clap(long, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    #[clap(long)]
    save_file: Option<PathBuf>,

    #[command(subcommand)]
    pub(crate) mode: Mode,
}

#[tokio::main]
async fn main() {
    //env_logger::init();

    let args = Cli::parse();
    
    //fmt().pretty().with_timer(time::UtcTime::rfc_3339()).init();

    let mut watcher = Watcher::<AlwaysReady>::default();
    metrics::register(&watcher);
    watcher.start_server(args.observability_address).await;

    let mut client_config = supermusr_common::generate_kafka_client_config(
        &args.broker,
        &args.username,
        &args.password,
    );

    let producer: FutureProducer = client_config
        .set("linger.ms", "0")
        .create()
        .expect("Kafka Producer should be created");
    
    let consumer: StreamConsumer = client_config
        .set("group.id", &args.consumer_group)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()
        .expect("Kafka Consumer should be created");


    consumer
        .subscribe(&[&args.trace_topic])
        .expect("Kafka Consumer should subscribe to trace-topic");

    let mut timer = StatTimer::new(500,1500);

    loop {
        if timer.has_finished() {
            let (stats1,stats2) = timer.calculate_stats();
            stats1.print();
            stats2.print();
            producer.flush(Timeout::After(Duration::from_millis(1000)))
                .expect("Messages Flush");
            return;
        }
        match consumer.recv().await {
            Ok(m) => {
                //timer_suite.unpack.record();
                timer.begin_record();
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
                        let bytes_in = payload.len();
                        metrics::MESSAGES_RECEIVED
                            .get_or_create(&metrics::MessagesReceivedLabels::new(
                                metrics::MessageKind::Trace,
                            ))
                            .inc();
                        let headers = m.headers()
                            .map(|h|h.detach())
                            .unwrap_or_default()
                            .insert(Header { key : "trace-to-events time_ns", value : Some(&[0]) });
                        match root_as_digitizer_analog_trace_message(payload) {
                            Ok(thing) => {
                                let mut fbb = FlatBufferBuilder::new();
                                processing::process(
                                    &mut fbb,
                                    &thing,
                                    &args.mode,
                                    args.save_file.as_deref(),
                                );
                                // End Timer
                                let future = producer.send_result(
                                    FutureRecord::to(&args.event_topic)
                                        .payload(fbb.finished_data())
                                        .headers(headers)
                                        .key("test")
                                )
                                .expect("Producer sends");
                                tokio::spawn(async {
                                    match future.await {
                                        Ok(_) => {
                                            log::trace!("Published event message");
                                            metrics::MESSAGES_PROCESSED.inc();
                                        }
                                        Err(e) => {
                                            log::error!("{:?}", e);
                                            metrics::FAILURES
                                                .get_or_create(&metrics::FailureLabels::new(
                                                    metrics::FailureKind::KafkaPublishFailed,
                                                ))
                                                .inc();
                                        }
                                    }
                                });
                                let bytes_out = fbb.finished_data().len();
                                fbb.reset();
                                // End Timer
                                timer.end_record( Data {bytes_in, bytes_out} );
                            }
                            Err(e) => {
                                log::warn!("Failed to parse message: {}", e);
                                metrics::FAILURES
                                    .get_or_create(&metrics::FailureLabels::new(
                                        metrics::FailureKind::UnableToDecodeMessage,
                                    ))
                                    .inc();
                            }
                        }
                    } else {
                        log::warn!("Unexpected message type on topic \"{}\"", m.topic());
                        metrics::MESSAGES_RECEIVED
                            .get_or_create(&metrics::MessagesReceivedLabels::new(
                                metrics::MessageKind::Unknown,
                            ))
                            .inc();
                    }
                }
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
            Err(e) => {
                log::warn!("Kafka error: {}", e);
            },
        }
    }
}
