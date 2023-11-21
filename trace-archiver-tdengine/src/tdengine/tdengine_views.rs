use anyhow::Result;
use std::iter::{once, repeat};

use itertools::Itertools;

use taos::{taos_query::common::views::TimestampView, ColumnView};

use common::{Intensity, Channel};

use super::{TDEngineError, TraceMessageErrorCode};

use super::{error_reporter::TDEngineErrorReporter, framedata::FrameData};

/// Creates a timestamp view from the current frame_data object
pub(super) fn create_timestamp_views(
    frame_data: &[FrameData],
) -> Result<(TimestampView, TimestampView)> {
    let mut frame_timestamp = Vec::<i64>::new();
    let mut sample_timestamp = Vec::<i64>::new();
    for fd in frame_data {
        use TraceMessageErrorCode as Code;
        use TDEngineError as TDErr;
        let frame_timestamp_ns = fd.timestamp
            .timestamp_nanos_opt()
            .ok_or(TDErr::TraceMessage(Code::TimestampMissing))?;
        let sample_time_ns = fd.sample_time
            .num_nanoseconds()
            .ok_or(TDErr::TraceMessage(Code::SampleTimeMissing))?;

        assert_ne!(sample_time_ns,0);

        for i in 0..fd.num_samples as i64 {
            frame_timestamp.push(frame_timestamp_ns);
            sample_timestamp.push(frame_timestamp_ns + sample_time_ns * i);
        }
    }

    // Create the views
    Ok((TimestampView::from_nanos(frame_timestamp),TimestampView::from_nanos(sample_timestamp)))
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
    let (frame_timestamp_view, timestamp_view) = {
        let (frame_timestamp_view, timestamp_view) = create_timestamp_views(frame_data)?;
        (ColumnView::Timestamp(frame_timestamp_view),ColumnView::Timestamp(timestamp_view))
    };

    let num_batch_samples = frame_data.iter().map(|f|f.num_samples).sum();
    let channel_voltage : Vec<_> = (0..num_channels).map(|c|{
        let mut voltage = Vec::<Intensity>::with_capacity(num_batch_samples);
        for fd in frame_data {
            voltage.extend(fd.trace_data[c].iter());
        }
        voltage
    }).collect();
    
    //let mut i = timestamp_view.iter();
    //println!("{0:?} : {1:?}", i.next().unwrap(), i.next().unwrap());

    
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
    num_channels : usize,
    frame_data: &[FrameData],
    error: &TDEngineErrorReporter,
) -> Result<Vec<ColumnView>> {
    let mut timestamp = Vec::<i64>::new();
    let mut num_samples = Vec::<u32>::new();
    let mut sample_rate = Vec::<u32>::new();
    let mut frame_number = Vec::<u32>::new();
    let mut error = Vec::<u32>::new();
    let mut channel_id = vec![Vec::<Channel>::new(); num_channels];
    for fd in frame_data {
        for c in 0..num_channels {
            channel_id[c].push(fd.channel_index[c]);
        }

        use TraceMessageErrorCode as Code;
        use TDEngineError as TDErr;
        timestamp.push(fd.timestamp
            .timestamp_nanos_opt()
            .ok_or(TDErr::TraceMessage(Code::CannotCalcMeasurementTime))?
        );
        num_samples.push(fd.num_samples as u32);
        sample_rate.push(fd.sample_rate as u32);
        frame_number.push(fd.frame_number);
        error.push(0);
    }
    Ok([
        ColumnView::Timestamp(TimestampView::from_nanos(timestamp)),
        ColumnView::from_unsigned_ints(num_samples),
        ColumnView::from_unsigned_ints(sample_rate),
        ColumnView::from_unsigned_ints(frame_number), 
        ColumnView::from_unsigned_ints(error),
    ]
    .into_iter()
    .chain(channel_id
        .into_iter()
        .map(|c|ColumnView::from_unsigned_ints(c))
    )
    .collect_vec())
}
