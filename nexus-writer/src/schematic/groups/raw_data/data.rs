use std::str::FromStr;

use chrono::{DateTime, Utc};
use hdf5::{Dataset, Group};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_pl72_run_start_generated::RunStart,
};

use crate::{
    nexus::{NexusSettings, RunParameters},
    schematic::{
        elements::{
            attribute::NexusAttribute,
            dataset::{NexusDataset, NexusDatasetResize},
            NexusBuildable, NexusDataHolder, NexusDataHolderAppendable,
            NexusDataHolderScalarMutable, NexusDatasetDef, NexusError, NexusGroupDef,
            NexusHandleMessage, NexusHandleMessageWithContext, NexusPushMessage, NexusUnits,
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
            offset: NexusAttribute::begin("offset").finish_with_auto_default(),
        }
    }
}

impl<'a> NexusHandleMessage<(&FrameAssembledEventListMessage<'a>, usize), Dataset, u64>
    for EventTimeZeroAttributes
{
    fn handle_message(
        &mut self,
        (message, current_index): &(&FrameAssembledEventListMessage<'a>, usize),
        _dataset: &Dataset,
    ) -> Result<u64, NexusError> {
        let timestamp: DateTime<Utc> =
            (*message.metadata().timestamp().ok_or(NexusError::Unknown)?)
                .try_into()
                .map_err(|_| NexusError::Unknown)?;

        let time_zero = {
            if *current_index != 0 {
                let offset = DateTime::<Utc>::from_str(self.offset.read_scalar(_dataset)?.as_str())
                    .map_err(|_| NexusError::Unknown)?;

                (timestamp - offset)
                    .num_nanoseconds()
                    .ok_or(NexusError::Unknown)?
                    .try_into()
                    .map_err(|_| NexusError::Unknown)?
            } else {
                self.offset.write_scalar(
                    _dataset,
                    timestamp
                        .to_rfc3339()
                        .parse()
                        .map_err(|_| NexusError::Unknown)?,
                )?;

                u64::default()
            }
        };
        Ok(time_zero)
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
            event_id: NexusDataset::begin("event_id").finish_with_resizable(
                0,
                0,
                settings.eventlist_chunk_size,
            ),
            event_index: NexusDataset::begin("event_index").finish_with_resizable(
                0,
                0,
                settings.framelist_chunk_size,
            ),
            event_time_offset: NexusDataset::begin("event_time_offset").finish_with_resizable(
                0,
                0,
                settings.eventlist_chunk_size,
            ),
            event_time_zero: NexusDataset::begin("event_time_zero").finish_with_resizable(
                0,
                0,
                settings.framelist_chunk_size,
            ),
            event_period_number: NexusDataset::begin("event_period_number").finish_with_resizable(
                0,
                0,
                settings.framelist_chunk_size,
            ),
            event_pulse_height: NexusDataset::begin("event_pulse_height").finish_with_resizable(
                0.0,
                0,
                settings.eventlist_chunk_size,
            ),
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
        let current_index = self.event_id.get_size(parent)?;
        self.event_id.append(
            parent,
            &message
                .channel()
                .ok_or(NexusError::Unknown)?
                .iter()
                .collect::<Vec<_>>(),
        )?;

        //  event_time_offset
        self.event_time_offset.append(
            parent,
            &message
                .time()
                .ok_or(NexusError::Unknown)?
                .iter()
                .collect::<Vec<_>>(),
        )?;
        self.event_time_offset.close_hdf5();

        //  event_pulse_height
        self.event_pulse_height.append(
            parent,
            &message
                .voltage()
                .ok_or(NexusError::Unknown)?
                .iter()
                .map(From::from)
                .collect::<Vec<_>>(),
        )?;

        //  event_index
        self.event_index.append(parent, &[current_index])?;

        //  event_period_number
        self.event_period_number
            .append(parent, &[message.metadata().period_number()])?;

        //  event_time_zero
        let time_zero = self
            .event_time_zero
            .push_message(&(message, current_index), parent)?;

        self.event_time_zero.append(parent, &[time_zero])?;
        Ok(())
    }
}
