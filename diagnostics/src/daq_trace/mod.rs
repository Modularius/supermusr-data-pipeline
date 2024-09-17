mod app;
mod ui;

use self::app::App;
use self::ui::ui;
use super::DaqTraceOpts;
use chrono::{DateTime, Utc};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::{prelude::CrosstermBackend, Terminal};
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use supermusr_common::{Channel, Intensity};
use supermusr_streaming_types::flatbuffers::Vector;
use std::collections::{HashMap, VecDeque};
use std::{
    io,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use supermusr_streaming_types::dat2_digitizer_analog_trace_v2_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message, DigitizerAnalogTraceMessage
};
use tokio::task;
use tokio::time::sleep;
use tracing::{debug, info, warn};

type DigitiserDataHashMap = Arc<Mutex<HashMap<u8, DigitiserData>>>;

enum Event<I> {
    Input(I),
    Tick,
}

pub struct TraceStats {
    window_ratio: f64,
    num_windows: usize,
    num_mean_values: usize,
    mean_values: VecDeque<f64>,
}

impl TraceStats {
    fn new(window_ratio: f64, num_windows: usize, num_mean_values: usize) -> Self {
        Self {
            window_ratio,
            num_windows,
            num_mean_values,
            mean_values: VecDeque::with_capacity(num_mean_values),
        }
    }
    fn get_subtrace_mean(&self, voltage: &Vector<Intensity>) -> f64 {
        let size = voltage.len();
        let window_size = (self.window_ratio*size as f64) as usize;
        (0..self.num_windows).map(|i| {
            let window_pos = size/self.num_windows
                + i*window_size
                + rand::random::<usize>() % window_size;
            voltage.iter()
                .map(f64::from)
                .skip(window_pos)
                .take(window_size)
                .sum::<f64>()
        })
        .sum::<f64>()/(window_size * self.num_windows) as f64
    }
    fn push_trace(&mut self, data: &DigitizerAnalogTraceMessage<'_>) {
        if let Some(channels) = data.channels() {
            let channel = channels.get(rand::random::<usize>() % channels.len());
            if let Some(voltage) = channel.voltage() {
                self.mean_values.push_back(self.get_subtrace_mean(&voltage));
                if self.mean_values.len() >= self.num_mean_values {
                    let _ = self.mean_values.pop_front();
                }
            }
        }
    }
    pub fn max(&self) -> Option<f64> {
        self.mean_values.iter().copied().reduce(f64::max)
    }
    pub fn min(&self) -> Option<f64> {
        self.mean_values.iter().copied().reduce(f64::min)
    }
}

/// Holds required data for a specific digitiser.
pub struct DigitiserData {
    pub msg_count: usize,
    last_msg_count: usize,
    pub msg_rate: f64,
    pub first_msg_timestamp: Option<DateTime<Utc>>,
    pub last_msg_timestamp: Option<DateTime<Utc>>,
    pub last_msg_frame: u32,
    pub num_channels_present: usize,
    pub channels_present: Option<Vec<Channel>>,
    pub has_num_channels_changed: bool,
    pub num_samples_in_first_channel: usize,
    pub is_num_samples_identical: bool,
    pub has_num_samples_changed: bool,
    pub bad_frame_count: usize,
    pub mean_value: TraceStats,
}

impl DigitiserData {
    /// Create a new instance with default values.
    pub fn new(
        timestamp: Option<DateTime<Utc>>,
        frame: u32,
        num_channels_present: usize,
        num_samples_in_first_channel: usize,
        is_num_samples_identical: bool,
    ) -> Self {
        DigitiserData {
            msg_count: 1,
            msg_rate: 0 as f64,
            last_msg_count: 1,
            first_msg_timestamp: timestamp,
            last_msg_timestamp: timestamp,
            last_msg_frame: frame,
            num_channels_present,
            channels_present : None,
            has_num_channels_changed: false,
            num_samples_in_first_channel,
            is_num_samples_identical,
            has_num_samples_changed: false,
            bad_frame_count: 0,
            mean_value: TraceStats::new(0.025, 2, 5),
        }
    }
}

/// Holds required data for a specific digitiser.
pub struct ChannelData {
    pub msg_count: usize,
    last_msg_count: usize,
    pub msg_rate: f64,
    pub first_msg_timestamp: Option<DateTime<Utc>>,
    pub last_msg_timestamp: Option<DateTime<Utc>>,
    pub last_msg_frame: u32,
    pub num_channels_present: usize,
    pub channels_present: Option<Vec<Channel>>,
    pub has_num_channels_changed: bool,
    pub num_samples_in_first_channel: usize,
    pub is_num_samples_identical: bool,
    pub has_num_samples_changed: bool,
    pub bad_frame_count: usize,
    pub mean_value: TraceStats,
}

impl ChannelData {
    /// Create a new instance with default values.
    pub fn new(
        timestamp: Option<DateTime<Utc>>,
        frame: u32,
        num_channels_present: usize,
        num_samples_in_first_channel: usize,
        is_num_samples_identical: bool,
    ) -> Self {
        ChannelData {
            msg_count: 1,
            msg_rate: 0 as f64,
            last_msg_count: 1,
            first_msg_timestamp: timestamp,
            last_msg_timestamp: timestamp,
            last_msg_frame: frame,
            num_channels_present,
            channels_present : None,
            has_num_channels_changed: false,
            num_samples_in_first_channel,
            is_num_samples_identical,
            has_num_samples_changed: false,
            bad_frame_count: 0,
            mean_value: TraceStats::new(0.025, 2, 5),
        }
    }
}

// Trace topic diagnostic tool
pub(crate) async fn run(args: DaqTraceOpts) -> anyhow::Result<()> {
    let kafka_opts = args.common.common_kafka_options;

    let consumer: StreamConsumer = supermusr_common::generate_kafka_client_config(
        &kafka_opts.broker,
        &kafka_opts.username,
        &kafka_opts.password,
    )
    .set("group.id", &args.common.consumer_group)
    .set("enable.partition.eof", "false")
    .set("session.timeout.ms", "6000")
    .set("enable.auto.commit", "false")
    .create()?;

    consumer.subscribe(&[&args.common.topic])?;

    // Set up terminal.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Set up app and common data.
    let mut app = App::new(&[
        "Digitiser ID",          // 1
        "#Msgs Received",        // 2
        "First Msg Timestamp",   // 3
        "Last Msg Timestamp",    // 4
        "Last Msg Frame",        // 5
        "Message Rate (Hz)",     // 6
        "#Bad Frames?",          // 7
    ]);

    // Set up app and common data.
    let mut app2 = App::new(&[
        "Channel Id",           // 1
        "#Samples",             // 2
        "#Samples Identical?",  // 3
        "#Samples Changed?",    // 4
        "Min Value",            // 5
        "Max Value",            // 6
    ]);

    let common_dig_data_map: DigitiserDataHashMap = Arc::new(Mutex::new(HashMap::new()));

    // Set up event polling.
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    // Event polling thread.
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).is_ok() {
                if let CEvent::Key(key) =
                    event::read().expect("should be able to read an event after a successful poll")
                {
                    tx.send(Event::Input(key))
                        .expect("should be able to send the key event via channel");
                }
            }

            if last_tick.elapsed() >= tick_rate && tx.send(Event::Tick).is_ok() {
                last_tick = Instant::now();
            }
        }
    });

    // Message polling thread.
    task::spawn(poll_kafka_msg(consumer, Arc::clone(&common_dig_data_map)));

    // Message rate calculation thread.
    task::spawn(update_message_rate(
        Arc::clone(&common_dig_data_map),
        args.message_rate_interval,
    ));

    // Run app.
    loop {
        // Poll events.
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => break,
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                KeyCode::Left => app2.next(),
                KeyCode::Right => app2.previous(),
                _ => (),
            },
            Event::Tick => (),
        }

        // Use the current data to regenerate the table body (may be inefficient to call every time).
        app.generate_table_body(Arc::clone(&common_dig_data_map));

        // Use the current data to regenerate the table body (may be inefficient to call every time).
        app2.generate_table_body2(Arc::clone(&common_dig_data_map));

        // Draw terminal using common data.
        terminal.draw(|frame| ui(frame, &mut app, &mut app2))?;
    }

    // Clean up terminal.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    terminal.clear()?;

    Ok(())
}

async fn update_message_rate(
    common_dig_data_map: DigitiserDataHashMap,
    recent_message_lifetime: u64,
) {
    loop {
        // Wait a set period of time before calculating average.
        sleep(Duration::from_secs(recent_message_lifetime)).await;
        let mut logged_data = common_dig_data_map
            .lock()
            .expect("should be able to lock common data");
        // Calculate message rate for each digitiser.
        for digitiser_data in logged_data.values_mut() {
            digitiser_data.msg_rate = (digitiser_data.msg_count - digitiser_data.last_msg_count)
                as f64
                / recent_message_lifetime as f64;
            digitiser_data.last_msg_count = digitiser_data.msg_count;
        }
    }
}

/// Poll kafka messages and update digitiser data.
async fn poll_kafka_msg(consumer: StreamConsumer, common_dig_data_map: DigitiserDataHashMap) {
    loop {
        match consumer.recv().await {
            Err(e) => warn!("Kafka error: {}", e),
            Ok(msg) => {
                debug!(
                    "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    msg.key(),
                    msg.topic(),
                    msg.partition(),
                    msg.offset(),
                    msg.timestamp()
                );

                if let Some(payload) = msg.payload() {
                    if digitizer_analog_trace_message_buffer_has_identifier(payload) {
                        match root_as_digitizer_analog_trace_message(payload) {
                            Ok(data) => {
                                let frame_number = data.metadata().frame_number();

                                let num_channels_present = match data.channels() {
                                    Some(c) => c.len(),
                                    None => 0,
                                };

                                let num_samples_in_first_channel = match data.channels() {
                                    Some(c) => match c.get(0).voltage() {
                                        Some(v) => v.len(),
                                        None => 0,
                                    },
                                    None => 0,
                                };

                                let is_num_samples_identical = match data.channels() {
                                    Some(c) => || -> bool {
                                        for trace in c.iter() {
                                            let num_samples = match trace.voltage() {
                                                Some(v) => v.len(),
                                                None => 0,
                                            };

                                            if num_samples != num_samples_in_first_channel {
                                                return false;
                                            }
                                        }
                                        true
                                    }(),
                                    None => false,
                                };

                                let timestamp = data
                                    .metadata()
                                    .timestamp()
                                    .copied()
                                    .and_then(|t| t.try_into().ok());

                                let id = data.digitizer_id();
                                {
                                    let mut logged_data = common_dig_data_map
                                        .lock()
                                        .expect("sound be able to lock common data");
                                    logged_data
                                        .entry(id)
                                        .and_modify(|d| {
                                            d.msg_count += 1;

                                            d.last_msg_timestamp = timestamp;
                                            d.last_msg_frame = frame_number;

                                            if timestamp.is_none() {
                                                d.bad_frame_count += 1;
                                            }

                                            let num_channels = match data.channels() {
                                                Some(c) => c.len(),
                                                None => 0,
                                            };
                                            if !d.has_num_channels_changed {
                                                d.has_num_channels_changed =
                                                    num_channels != d.num_channels_present;
                                            }
                                            d.num_channels_present = num_channels;
                                            if !d.has_num_channels_changed {
                                                d.has_num_samples_changed =
                                                    num_samples_in_first_channel
                                                        != d.num_samples_in_first_channel;
                                            }
                                            d.channels_present = data.channels().map(|cs|cs.iter().map(|c|c.channel()).collect());
                                            d.num_samples_in_first_channel =
                                                num_samples_in_first_channel;
                                            d.is_num_samples_identical = is_num_samples_identical;


                                            // digitiser sample mean
                                            d.mean_value.push_trace(&data);
                                        })
                                        .or_insert(DigitiserData::new(
                                            timestamp,
                                            frame_number,
                                            num_channels_present,
                                            num_samples_in_first_channel,
                                            is_num_samples_identical,
                                        ));
                                };

                                info!(
                                    "Trace packet: dig. ID: {}, metadata: {:?}",
                                    data.digitizer_id(),
                                    data.metadata()
                                );
                            }
                            Err(e) => {
                                warn!("Failed to parse message: {}", e);
                            }
                        }
                    } else {
                        warn!("Unexpected message type on topic \"{}\"", msg.topic());
                    }
                }

                let _ = consumer.commit_message(&msg, CommitMode::Async);
            }
        };
    }
}
