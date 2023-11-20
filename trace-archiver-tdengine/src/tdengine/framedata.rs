use std::{ops::Div, iter::repeat};

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use common::{DigitizerId, FrameNumber, Intensity, Channel};
use streaming_types::dat1_digitizer_analog_trace_v1_generated::{DigitizerAnalogTraceMessage, ChannelTrace};

use super::{TDEngineError, TraceMessageErrorCode};

/// Stores and handles some of the data obtained from a DigitizerAnalogTraceMessage message.
/// # Fields
/// * `timestamp` - The timestamp of the current frame.
/// * `frame_number` - The frame number of the current frame.
/// * `digitizer_id` - The id of the digitizer.
/// * `sample_time` - The duration of each sample in the current frame.
#[derive(Clone)]
pub(super) struct FrameData {
    pub timestamp: DateTime<Utc>,
    pub digitizer_id: DigitizerId,
    pub frame_number: FrameNumber,
    pub num_channels: usize,
    pub num_samples: usize,
    pub sample_time: Duration,
    pub sample_rate: u64,
    pub trace_data: Vec<Vec<Intensity>>,
    pub channel_index: Vec<Channel>,
}
impl Default for FrameData {
    fn default() -> Self {
        FrameData {
            timestamp: DateTime::<Utc>::default(),
            digitizer_id: DigitizerId::default(),
            frame_number: FrameNumber::default(),
            num_channels: usize::default(),
            num_samples: usize::default(),
            sample_time: Duration::nanoseconds(0),
            sample_rate: u64::default(),
            trace_data: Vec::new(),
            channel_index: Vec::new(),
        }
    }
}
impl FrameData {
    /// Extracts some of the data from a DigitizerAnalogTraceMessage message.
    /// Note that no channel trace data is extracted.
    /// # Arguments
    /// * `message` - A reference to a DigitizerAnalogTraceMessage message.
    /// # Returns
    /// An emtpy result, or an error.
    pub(super) fn init(&mut self, message: &DigitizerAnalogTraceMessage) -> Result<()> {
        //  Obtain the timestamp, and error check
        self.timestamp = (*message
            .metadata()
            .timestamp()
            .ok_or(TDEngineError::TraceMessage(
                TraceMessageErrorCode::TimestampMissing,
            ))?)
        .into();

        //  Obtain the detector data
        self.digitizer_id = message.digitizer_id();
        self.frame_number = message.metadata().frame_number();

        // Obtain the sample rate and calculate the sample time (ns)
        self.sample_rate = message.sample_rate();
        if self.sample_rate == 0 {
            Err(TDEngineError::TraceMessage(
                TraceMessageErrorCode::SampleRateZero,
            ))?;
        }
        self.sample_time = Duration::nanoseconds(1_000_000_000).div(self.sample_rate as i32);
        if self.sample_time.is_zero() {
            Err(TDEngineError::TraceMessage(
                TraceMessageErrorCode::SampleTimeZero,
            ))?;
        }

        if message.channels().is_none() {
            Err(TDEngineError::TraceMessage(
                TraceMessageErrorCode::ChannelDataNull,
            ))?;
        }

        // Get the maximum number of samples from the channels,
        // Note this does not perform any tests on the channels.
        self.num_samples = message
            .channels()
            .unwrap()
            .iter()
            .filter_map(|c| c.voltage())
            .map(|v| v.len())
            .max()
            .unwrap_or_default();
        Ok(())
    }

    pub(super) fn get_table_name(&self) -> String {
        format!("d{0}", self.digitizer_id)
    }
    pub(super) fn get_frame_table_name(&self) -> String {
        format!("m{0}", self.digitizer_id)
    }

    pub(super) fn extract_channel_data(&mut self, message: &DigitizerAnalogTraceMessage) -> Result<()> {
        let null_channel_samples = repeat(Intensity::default()).take(self.num_samples);
        let channel_padding = repeat(null_channel_samples)
            .take(self.num_channels)
            .skip(message.channels().unwrap().len())
            .map(|v| v.collect());
        
        self.trace_data = message
            .channels()
            .unwrap()
            .iter()
            .take(self.num_channels) // Cap the channel list at the given channel count
            .map(|c|self.create_voltage_values_from_channel_trace(&c))
            .chain(channel_padding)
            .collect();
        
        let channel_padding = repeat(0)
            .take(self.num_channels)
            .skip(message.channels().unwrap().len());
        self.channel_index = message
            .channels()
            .unwrap()
            .iter()
            .take(self.num_channels) // Cap the channel list at the given channel count
            .map(|c|c.channel())
            .chain(channel_padding)
            .collect();
        Ok(())
    }

    /// Calculates the timestamp of a particular measurement relative to the timestamp of this frame.
    /// # Arguments
    /// * `measurement_number` - The measurement number to calculate the timestamp for.
    /// # Returns
    /// A `DateTime<Utc>` representing the measurement timestamp.
    pub(super) fn calc_measurement_time(&self, measurment_number: usize) -> DateTime<Utc> {
        self.timestamp + self.sample_time * measurment_number as i32
    }

    
    /// Creates a vector of intensity values of size equal to the correct number of samples
    /// These are extracted from the channel trace if available. If not then a vector of zero
    /// Values is created
    /// #Arguments
    /// *channel - a reference to the channel trace to extract from
    /// #Return
    /// A vector of intensities
    fn create_voltage_values_from_channel_trace<'a>(
        &self,
        channel: &'a ChannelTrace,
    ) -> Vec<Intensity> {
        let voltage = channel.voltage().unwrap_or_default();
        let padding = repeat(Intensity::default())
            .take(self.num_samples)
            .skip(voltage.len());

        voltage.iter().chain(padding).collect()
    }
}
