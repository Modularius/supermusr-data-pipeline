use std::str::FromStr;

use chrono::{DateTime, Duration, Utc};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_pl72_run_start_generated::RunStart,
};

use crate::schematic::{
    elements::{
        attribute::NexusAttribute,
        dataset::{NexusDataset, NexusDatasetResize},
        NexusBuildable, NexusBuilderFinished, NexusDataHolderAppendable,
        NexusDataHolderScalarMutable, NexusDatasetDef, NexusError, NexusGroupDef, NexusPushMessage,
        NexusUnits,
    },
    nexus_class, H5String,
};

#[derive(Clone)]
struct EventTimeOffsetAttributes {}

impl NexusDatasetDef for EventTimeOffsetAttributes {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Nanoseconds);

    fn new() -> Self {
        Self {}
    }
}

#[derive(Clone)]
struct EventTimeZeroAttributes {
    offset: NexusAttribute<H5String>,
}

impl NexusDatasetDef for EventTimeZeroAttributes {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Nanoseconds);

    fn new() -> Self {
        Self {
            offset: NexusAttribute::begin("offset")
                .default_value(Default::default())
                .finish(),
        }
    }
}

impl<'a> NexusPushMessage<FrameAssembledEventListMessage<'a>> for EventTimeZeroAttributes {
    type MessageType = FrameAssembledEventListMessage<'a>;

    fn push_message(&self, message: &Self::MessageType) -> Result<(), NexusError> {
        let timestamp: DateTime<Utc> =
            (*message.metadata().timestamp().ok_or(NexusError::Unknown)?)
                .try_into()
                .map_err(|_| NexusError::Unknown)?;
        self.offset.write_scalar(
            timestamp
                .to_rfc3339()
                .parse()
                .map_err(|_| NexusError::Unknown)?,
        )
    }
}

pub(super) struct Data {
    event_id: NexusDatasetResize<u32>,
    event_index: NexusDatasetResize<usize>,
    event_time_offset: NexusDatasetResize<u32, EventTimeOffsetAttributes>,
    event_time_zero: NexusDatasetResize<u64, EventTimeZeroAttributes>,
    event_period_number: NexusDatasetResize<u64>,
    event_pulse_height: NexusDatasetResize<f64>,
}

impl NexusGroupDef for Data {
    const CLASS_NAME: &'static str = nexus_class::EVENT_DATA;

    fn new() -> Self {
        Self {
            event_id: NexusDataset::begin("event_id")
                .resizable(0, 0, 128)
                .finish(),
            event_index: NexusDataset::begin("event_index")
                .resizable(0, 0, 128)
                .finish(),
            event_time_offset: NexusDataset::begin("event_time_offset")
                .resizable(0, 0, 128)
                .finish(),
            event_time_zero: NexusDataset::begin("event_time_zero")
                .resizable(0, 0, 128)
                .finish(),
            event_period_number: NexusDataset::begin("event_period_number")
                .resizable(0, 0, 128)
                .finish(),
            event_pulse_height: NexusDataset::begin("event_pulse_height")
                .resizable(0.0, 0, 128)
                .finish(),
        }
    }
}

impl<'a> NexusPushMessage<RunStart<'a>> for Data {
    type MessageType = RunStart<'a>;

    fn push_message(&self, message: &Self::MessageType) -> Result<(), NexusError> {
        //let timestamp = DateTime::<Utc>::from_timestamp_millis(i64::try_from(message.start_time())?).ok_or(anyhow::anyhow!("Millisecond error"))?;
        //self.event_time_zero.attributes(|attributes|Ok(attributes.offset.write_scalar(timestamp.to_rfc3339().parse()?)?))?;
        Ok(())
    }
}

impl<'a> NexusPushMessage<FrameAssembledEventListMessage<'a>> for Data {
    type MessageType = FrameAssembledEventListMessage<'a>;

    fn push_message(&self, message: &Self::MessageType) -> Result<(), NexusError> {
        // Here is where we extend the datasets
        let current_index = self.event_id.get_size()?;
        self.event_id.append(
            &message
                .channel()
                .ok_or(NexusError::Unknown)?
                .iter()
                .collect::<Vec<_>>(),
        )?;
        self.event_time_offset.append(
            &message
                .time()
                .ok_or(NexusError::Unknown)?
                .iter()
                .collect::<Vec<_>>(),
        )?;
        self.event_pulse_height.append(
            &message
                .voltage()
                .ok_or(NexusError::Unknown)?
                .iter()
                .map(From::from)
                .collect::<Vec<_>>(),
        )?;

        self.event_index.append(&[current_index])?;
        self.event_period_number
            .append(&[message.metadata().period_number()])?;

        let timestamp: DateTime<Utc> =
            (*message.metadata().timestamp().ok_or(NexusError::Unknown)?)
                .try_into()
                .map_err(|_| NexusError::Unknown)?;

        let time_zero = {
            if current_index != 0 {
                let offset_string = self
                    .event_time_zero
                    .attribute(|attributes| &attributes.offset).read_scalar()?;
                let offset = DateTime::<Utc>::from_str(offset_string.as_str())
                    .map_err(|_| NexusError::Unknown)?;
                timestamp - offset
            } else {
                self.event_time_zero.push_message(message)?;
                Duration::zero()
            }
        }
        .num_nanoseconds()
        .ok_or(NexusError::Unknown)? as u64;

        self.event_time_zero.append(&[time_zero])?;
        Ok(())
    }
}
