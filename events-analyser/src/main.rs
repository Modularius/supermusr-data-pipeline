use chrono::{DateTime, Utc};
use clap::Parser;
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::{BorrowedMessage, Headers, Message, ToBytes},
};
use supermusr_common::{Channel, DigitizerId, FrameNumber, Intensity, Time};
use std::{collections::HashMap, fmt::Debug, fs::File, io::Write};
use supermusr_streaming_types::dev1_digitizer_event_v1_generated::{digitizer_event_list_message_buffer_has_identifier, root_as_digitizer_event_list_message};

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
    simulated_events_topic: String,
}

#[derive(Default, Clone, PartialEq, Eq, Hash)]
struct MessageKey {
    ts: DateTime<Utc>,
    digitiser_id: DigitizerId,
    frame_number : FrameNumber,
}

type ChannelList = HashMap<Channel,EventList>;

#[derive(Default)]
struct EventList {
    voltage: Vec<Intensity>,
    time : Vec<Time>,
}

#[tokio::main]
async fn main() {
    env_logger::init();

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

    consumer
        .subscribe(&[&args.trace_to_events_topic, &args.simulated_events_topic])
        .expect("Kafka Consumer should subscribe to given topics");

    let mut event_pairs = HashMap::new();
    File::options().truncate(true).write(true).create(true).open("timings.csv").unwrap();

    loop {
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
                    if digitizer_event_list_message_buffer_has_identifier(payload) {
                        match root_as_digitizer_event_list_message(payload) {
                            Ok(thing) => {
                                let key = MessageKey {
                                    ts: (*thing.metadata().timestamp().unwrap()).into(),
                                    digitiser_id: thing.digitizer_id(),
                                    frame_number: thing.metadata().frame_number(),
                                };
                                let event_pair = event_pairs.entry(key.clone()).or_insert((None,None));

                                let mut list = ChannelList::default();
                                for (i,c) in thing.channel().unwrap().iter().enumerate() {
                                    let event_list = list.entry(c).or_insert(EventList::default());
                                    event_list.time.push(thing.time().unwrap().get(i));
                                    event_list.voltage.push(thing.voltage().unwrap().get(i));
                                }
                                if m.topic() == args.trace_to_events_topic {
                                    event_pair.0 = Some(list);
                                    let headers = extract_header(&m);
                                    if let Some(time_ns) = headers.get("trace-to-events: time_ns") {
                                        let file = File::options()
                                            .append(true)
                                            .create(true)
                                            .open("timings.csv")
                                            .unwrap();
                                        writeln!(&file, "{0},{1},{2}", DateTime::<Utc>::from(*thing.metadata().timestamp().unwrap()), key.frame_number, time_ns).unwrap();
                                    }
                                } else if m.topic() == args.simulated_events_topic {
                                    event_pair.1 = Some(list);
                                }
                                if let (Some(event1), Some(event2)) = event_pair {
                                    perform_analysis(event1,event2);
                                    event_pairs.remove(&key);
                                } 
                            }
                            Err(e) => {
                                log::warn!("Failed to parse message: {}", e);
                            }
                        }
                    } else {
                        log::warn!("Unexpected message type on topic \"{}\"", m.topic());
                    }
                }

                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
            Err(e) => log::warn!("Kafka error: {}", e),
        };
    }
}


fn perform_analysis(list1 : &ChannelList, list2 : &ChannelList) {
    println!("Performing Event List Analysis");
    if list1.keys().collect::<Vec<_>>() != list2.keys().collect::<Vec<_>>() {
        println!("Channel mismatch. Returning");
        return;
    }

    for c in list1.keys() {
        println!("Analysing Channel {c}");
        let event_list1 = list1.get(c).unwrap();
        let event_list2 = list2.get(c).unwrap();
        println!("Number of events in, list 1 = {0}, list 2 = {1}",
            event_list1.time.len(),
            event_list2.time.len()
        );
        
        println!("Lifetime estimator for, list 1 = {0}, list 2 = {1}",
            (event_list1.time.len() as f64 - 2.0)/event_list1.time.iter().sum::<Time>() as f64,
            (event_list2.time.len() as f64 - 2.0)/event_list2.time.iter().sum::<Time>() as f64
        );
    }
}

fn extract_header(m : &BorrowedMessage) -> HashMap::<String, String>{
    let mut map = HashMap::<String, String>::new(); 
    if let Some(headers) = m.headers().map(|h|h.detach()) {
        for h in headers.iter() {
            let val = String::from_utf8(h.value.unwrap().to_owned()).unwrap();
            map.insert(h.key.to_owned(), val);
        }
    }
    map
}