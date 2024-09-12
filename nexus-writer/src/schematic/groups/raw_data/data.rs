use std::str::FromStr;

use chrono::{DateTime, Duration, Utc};
use hdf5::{Dataset, Group, Location};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_pl72_run_start_generated::RunStart,
};

use crate::{
    nexus::{NexusSettings, Run, RunParameters},
    schematic::{
        elements::{
            attribute::NexusAttribute,
            dataset::{NexusDataset, NexusDatasetResize},
            NexusBuildable, NexusBuilderFinished, NexusDataHolder, NexusDataHolderAppendable,
            NexusDataHolderScalarMutable, NexusDatasetDef, NexusError, NexusGroupDef,
            NexusHandleMessage, NexusHandleMessageWithContext, NexusPushMessage,
            NexusPushMessageWithContext, NexusUnits,
        },
        nexus_class, H5String,
    },
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

impl<'a> NexusHandleMessage<(&FrameAssembledEventListMessage<'a>, usize), Dataset, u64>
    for EventTimeZeroAttributes
{
    fn handle_message(
        &mut self,
        message: &(&FrameAssembledEventListMessage<'a>, usize),
        _dataset: &Dataset,
    ) -> Result<u64, NexusError> {
        let (message, current_index) = message;
        let timestamp: DateTime<Utc> =
            (*message.metadata().timestamp().ok_or(NexusError::Unknown)?)
                .try_into()
                .map_err(|_| NexusError::Unknown)?;

        Ok(if *current_index != 0 {
            let offset = DateTime::<Utc>::from_str(self.offset.read_scalar()?.as_str())
                .map_err(|_| NexusError::Unknown)?;
            timestamp - offset
        } else {
            self.offset.write_scalar(
                timestamp
                    .to_rfc3339()
                    .parse()
                    .map_err(|_| NexusError::Unknown)?,
            )?;
            Duration::zero()
        }
        .num_nanoseconds()
        .ok_or(NexusError::Unknown)? as u64)
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
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            event_id: NexusDataset::begin("event_id")
                .resizable(0, 0, settings.eventlist_chunk_size)
                .finish(),
            event_index: NexusDataset::begin("event_index")
                .resizable(0, 0, settings.framelist_chunk_size)
                .finish(),
            event_time_offset: NexusDataset::begin("event_time_offset")
                .resizable(0, 0, settings.eventlist_chunk_size)
                .finish(),
            event_time_zero: NexusDataset::begin("event_time_zero")
                .resizable(0, 0, settings.framelist_chunk_size)
                .finish(),
            event_period_number: NexusDataset::begin("event_period_number")
                .resizable(0, 0, settings.framelist_chunk_size)
                .finish(),
            event_pulse_height: NexusDataset::begin("event_pulse_height")
                .resizable(0.0, 0, settings.eventlist_chunk_size)
                .finish(),
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>> for Data {
    fn handle_message(&mut self, message: &RunStart<'a>, _group: &Group) -> Result<(), NexusError> {
        //let timestamp = DateTime::<Utc>::from_timestamp_millis(i64::try_from(message.start_time())?).ok_or(anyhow::anyhow!("Millisecond error"))?;
        //self.event_time_zero.attributes(|attributes|Ok(attributes.offset.write_scalar(timestamp.to_rfc3339().parse()?)?))?;
        Ok(())
    }
}

impl<'a> NexusHandleMessageWithContext<FrameAssembledEventListMessage<'a>> for Data {
    type Context = RunParameters;

    fn handle_message_with_context(
        &mut self,
        message: &FrameAssembledEventListMessage<'a>,
        parent: &Group,
        _params: &mut RunParameters,
    ) -> Result<(), NexusError> {
        // Here is where we extend the datasets

        //  event_id
        self.event_id.create_hdf5(&parent)?;
        let current_index = self.event_id.get_size()?;
        self.event_id.append(
            &message
                .channel()
                .ok_or(NexusError::Unknown)?
                .iter()
                .collect::<Vec<_>>(),
        )?;
        self.event_id.close_hdf5();

        //  event_time_offset
        self.event_time_offset.create_hdf5(&parent)?;
        self.event_time_offset.append(
            &message
                .time()
                .ok_or(NexusError::Unknown)?
                .iter()
                .collect::<Vec<_>>(),
        )?;
        self.event_time_offset.close_hdf5();

        //  event_pulse_height
        self.event_pulse_height.create_hdf5(&parent)?;
        self.event_pulse_height.append(
            &message
                .voltage()
                .ok_or(NexusError::Unknown)?
                .iter()
                .map(From::from)
                .collect::<Vec<_>>(),
        )?;
        self.event_pulse_height.close_hdf5();

        //  event_index
        self.event_index.create_hdf5(&parent)?;
        self.event_index.append(&[current_index])?;
        self.event_index.close_hdf5();

        //  event_period_number
        self.event_period_number.create_hdf5(&parent)?;
        self.event_period_number
            .append(&[message.metadata().period_number()])?;
        self.event_period_number.close_hdf5();

        //  event_time_zero
        let time_zero =
            self.event_time_zero
                .push_message(&(message,current_index), &parent)?;

        self.event_time_zero.append(&[time_zero])?;
        Ok(())
    }
}
