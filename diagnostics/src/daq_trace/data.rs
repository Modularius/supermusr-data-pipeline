use supermusr_streaming_types::dat2_digitizer_analog_trace_v2_generated::DigitizerAnalogTraceMessage;
use supermusr_common::{Channel, Intensity};
use supermusr_streaming_types::flatbuffers::Vector;
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};

#[derive(Default, Clone)]
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
    pub(crate) fn push_trace(&mut self, data: &DigitizerAnalogTraceMessage<'_>) {
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

pub(crate) trait TableHeaders<const NUM_COLS : usize> {
    const TABLE_HEADERS : [&'static str; NUM_COLS];
}

/// Holds required data for a specific digitiser.
pub struct DigitiserData {
    
    pub msg_count: usize,
    pub last_msg_count: usize,
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
    pub channels: Vec<ChannelData>,
}

impl TableHeaders<7> for DigitiserData {
    const TABLE_HEADERS : [&'static str; 7] = [
        "Digitiser ID",          // 1
        "#Msgs Received",        // 2
        "First Msg Timestamp",   // 3
        "Last Msg Timestamp",    // 4
        "Last Msg Frame",        // 5
        "Message Rate (Hz)",     // 6
        "#Bad Frames?",          // 7
    ];
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
            channels: Default::default()
        }
    }
}

/// Holds required data for a specific channel.
#[derive(Default, Clone)]
pub struct ChannelData {
    pub id: Channel,
    pub num_samples: usize,
    pub has_num_samples_changed: bool,
    pub mean_value: TraceStats,
}

impl TableHeaders<5> for ChannelData {
    const TABLE_HEADERS : [&'static str; 5] = [
        "Channel Id",           // 1
        "#Samples",             // 2
        "#Samples Changed?",    // 3
        "Min Value",            // 4
        "Max Value",            // 5
    ];
}

impl ChannelData {
    /// Create a new instance with default values.
    pub fn new(
        id: Channel,
        num_samples: usize,
    ) -> Self {
        ChannelData {
            id,
            num_samples,
            mean_value: TraceStats::new(0.025, 2, 5),
            has_num_samples_changed: false,
        }
    }
}
