use anyhow::Result;
use std::iter::{once, repeat, Chain, Skip, Take, Repeat};

use itertools::Itertools;

use taos::{taos_query::common::views::TimestampView, ColumnView};

use common::{Intensity, Channel};
use streaming_types::{
    dat1_digitizer_analog_trace_v1_generated::ChannelTrace,
    flatbuffers::{ForwardsUOffset, Vector, VectorIter},
};

use super::{TDEngineError, TraceMessageErrorCode};

use super::{error_reporter::TDEngineErrorReporter, framedata::FrameData};

/// Creates a timestamp view from the current frame_data object
pub(super) fn create_timestamp_views(
    frame_data: &[FrameData],
) -> Result<(TimestampView, TimestampView)> {
    let mut frame_timestamp = Vec::<i64>::new();
    let mut sample_timestamp = Vec::<i64>::new();
    for fd in frame_data {
        let frame_timestamp_ns = fd.timestamp
            .timestamp_nanos_opt()
            .ok_or(TDEngineError::TraceMessage(
                TraceMessageErrorCode::TimestampMissing,
            ))?;
        let sample_time_ns = fd.sample_time
            .num_nanoseconds()
            .ok_or(TDEngineError::TraceMessage(
                TraceMessageErrorCode::SampleTimeMissing,
            ))?;
        for i in 0..fd.num_samples as i64 {
            frame_timestamp.push(frame_timestamp_ns);
            sample_timestamp.push(frame_timestamp_ns + sample_time_ns * i);
        }
    }

    // Create the views
    Ok((
        TimestampView::from_nanos(frame_timestamp),
        TimestampView::from_nanos(sample_timestamp)
    ))
}


/// CreateCreates a vector of column views which can be bound to a TDEngine statement
/// consisting of a timestamp view and the predefined number of channel views. If the
/// number of channel traces is greater than the predefined number then the surplus
/// channels are discarded. If the number of channel traces is insufficient then views
/// consisting of zero intensities are appended as neecessary.
/// #Arguments
/// *message - the DigitizerAnalogTraceMessage instance to extract from
/// #Return
/// A vector of column views
pub(super) fn create_column_views(num_channels: usize,
    frame_data: &[FrameData],
) -> Result<Vec<ColumnView>> {
    let (timestamp_view, frame_timestamp_view) = {
        let (timestamp_view, frame_timestamp_view) = create_timestamp_views(frame_data)?;
        (
            ColumnView::Timestamp(timestamp_view),
            ColumnView::Timestamp(frame_timestamp_view),
        )
    };

    let num_batch_samples = frame_data.iter().map(|f|f.num_samples).sum();
    let channel_voltage : Vec<_> = (0..num_channels).map(|c|{
        let mut voltage = Vec::<Intensity>::with_capacity(num_batch_samples);
        for fd in frame_data {
            voltage.extend(fd.trace_data[c].iter());
        }
        voltage
    }).collect();
    let channel_voltage_view = channel_voltage.into_iter().map(|c|ColumnView::from_unsigned_small_ints(c));

    Ok(once(timestamp_view)
        .chain(once(frame_timestamp_view))
        .chain(channel_voltage_view)
        .collect_vec())
}

/// Creates a vector of taos_query values which contain the tags to be used for the tdengine
/// statement.
/// #Arguments
/// *channels - a flatbuffers vector of ChannelTraces from which the tags are created
/// #Returns
/// A vector of taos_query values
pub(super) fn create_frame_column_views(
    frame_data: &[FrameData],
    error: &TDEngineErrorReporter,
) -> Result<Vec<ColumnView>> {
    let mut timestamp = Vec::<i64>::new();
    let mut num_samples = Vec::<u32>::new();
    let mut sample_rate = Vec::<u32>::new();
    let mut frame_number = Vec::<u32>::new();
    let mut error = Vec::<u32>::new();
    let mut channel_id = Vec::<Vec<Channel>>::new();
    for fd in frame_data {
        let channel_padding = repeat(0)
            .take(fd.num_channels)
            .skip(fd.channel_index.len());

        channel_id.push(fd.channel_index
            .iter()
            .map(|c|*c)
            .take(fd.num_channels) // Cap the channel list at the given channel count
            .chain(channel_padding)
            .collect()
        ); // Append any additional channels if needed

        timestamp.push(fd.calc_measurement_time(0)
            .timestamp_nanos_opt()
            .ok_or(TDEngineError::TraceMessage(TraceMessageErrorCode::CannotCalcMeasurementTime))?
        );
        num_samples.push(fd.num_samples as u32);
        sample_rate.push(fd.sample_rate as u32);
        frame_number.push(fd.frame_number);
        error.push(0);
    }
    Ok([
        ColumnView::from_nanos_timestamp(timestamp),
        ColumnView::from_unsigned_ints(num_samples),
        ColumnView::from_unsigned_ints(sample_rate),
        ColumnView::from_unsigned_ints(frame_number), 
    ]
    .into_iter()
    .chain(channel_id
        .into_iter()
        .map(|c|ColumnView::from_unsigned_ints(c))
    )
    .collect_vec())
}
