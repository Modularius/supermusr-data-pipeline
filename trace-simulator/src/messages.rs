
use anyhow::{Result, Error};
use itertools::Itertools;
//use std::ops::Range;
use chrono::Utc;
use core::ops::Range;
use flatbuffers::{FlatBufferBuilder, WIPOffset};
use rand::{random, rngs::ThreadRng, thread_rng, Rng};
use std::{ops::RangeInclusive, time::Duration};
use rdkafka::{producer::{FutureProducer, FutureRecord}, util::Timeout};

use common::{Channel, DigitizerId, FrameNumber, Intensity};
use streaming_types::{
    dat1_digitizer_analog_trace_v1_generated::{
        finish_digitizer_analog_trace_message_buffer, ChannelTrace, ChannelTraceArgs,
        DigitizerAnalogTraceMessage, DigitizerAnalogTraceMessageArgs,
    },
    frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args, GpsTime},
};

use crate::MessageGenerator;


/// Loads a FlatBufferBuilder with a new DigitizerAnalogTraceMessage instance with the present timestamp.
/// #Arguments
/// * `fbb` - A mutable reference to the FlatBufferBuilder to use.
/// * `frame_number` - The frame number to use.
/// * `digitizer_id` - The id of the digitizer to use.
/// * `measurements_per_frame` - The number of measurements to simulate in each channel.
/// * `num_channels` - The number of channels to simulate.
/// #Returns
/// A string result, or an error.



/// Loads a FlatBufferBuilder with a new DigitizerAnalogTraceMessage instance with a custom timestamp,
/// and a random frame number and digitizer id.
/// #Arguments
/// * `fbb` - A mutable reference to the FlatBufferBuilder to use.
/// * `time` - A `frame_metadata_v1_generated::GpsTime` instance containing the timestamp.
/// * `frame_number` - The upper and lower bounds from which to sample the frame number from.
/// * `digitizer_id` - The upper and lower bounds from which to sample the digitizer id from.
/// * `measurements_per_frame` - The number of measurements to simulate in each channel.
/// * `num_channels` - The number of channels to simulate.
/// #Returns
/// A string result, or an error.
pub fn create_partly_random_message(
    fbb: &mut FlatBufferBuilder<'_>,
    time: GpsTime,
    frame_number: RangeInclusive<FrameNumber>,
    digitizer_id: RangeInclusive<DigitizerId>,
    measurements_per_frame: usize,
    num_channels: usize,
    data: &impl MessageGenerator,
) -> Result<String, Error> {
    let mut rng = rand::thread_rng();
    let frame_number = rng.gen_range(frame_number);
    let digitizer_id = rng.gen_range(digitizer_id);
    MessageGenerator::create_message(fbb, time, frame_number, digitizer_id, measurements_per_frame, num_channels, data)
}

/// Loads a FlatBufferBuilder with a new DigitizerAnalogTraceMessage instance with a custom timestamp,
/// and all random parameters.
/// #Arguments
/// * `fbb` - A mutable reference to the FlatBufferBuilder to use.
/// * `time` - A `frame_metadata_v1_generated::GpsTime` instance containing the timestamp.
/// * `frame_number` - The upper and lower bounds from which to sample the frame number from.
/// * `digitizer_id` - The upper and lower bounds from which to sample the digitizer id from.
/// * `measurements_per_frame` - The upper and lower bounds from which to sample the number of measurements from.
/// * `num_channels` - The upper and lower bounds from which to sample the number of channels from.
/// #Returns
/// A string result, or an error.
pub fn create_random_message(
    fbb: &mut FlatBufferBuilder<'_>,
    time: GpsTime,
    frame_number: RangeInclusive<FrameNumber>,
    digitizer_id: RangeInclusive<DigitizerId>,
    measurements_per_frame: RangeInclusive<usize>,
    num_channels: RangeInclusive<usize>,
    generator: &impl MessageGenerator,
) -> Result<String, Error> {
    let mut rng = rand::thread_rng();
    let measurements_per_frame = rng.gen_range(measurements_per_frame);
    let num_channels = rng.gen_range(num_channels);
    create_partly_random_message(
        fbb,
        time,
        frame_number,
        digitizer_id,
        measurements_per_frame,
        num_channels,
        generator,
    )
}
