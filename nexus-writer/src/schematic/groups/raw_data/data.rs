use std::str::FromStr;

use chrono::{DateTime, Utc};
use hdf5::{Dataset, Group};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_pl72_run_start_generated::RunStart,
};

use crate::{
    elements::{
        attribute::{NexusAttribute, NexusAttributeMut},
        dataset::{NexusDataset, NexusDatasetResize},
        traits::{
            NexusAppendableDataHolder, NexusDataHolderScalarMutable, NexusDataHolderStringMutable,
            NexusDataHolderWithSize, NexusDatasetDef, NexusDatasetDefUnitsOnly, NexusGroupDef,
            NexusH5CreatableDataHolder, NexusHandleMessage, NexusPushMessage,
        },
        NexusUnits,
    },
    error::{NexusConversionError, NexusMissingError, NexusMissingEventlistError, NexusPushError},
    nexus::NexusSettings,
    schematic::{nexus_class, H5String},
};

/*
    Dataset: EventTimeOffset
*/
#[derive(Default, Clone)]
struct EventTimeOffset {}

impl NexusDatasetDefUnitsOnly for EventTimeOffset {
    const UNITS: NexusUnits = NexusUnits::Nanoseconds;
}

/*
    Dataset: EventTimeZero
*/
#[derive(Clone)]
struct EventTimeZero {
    offset: NexusAttributeMut<H5String>,
}

impl NexusDatasetDef for EventTimeZero {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Nanoseconds);

    fn new() -> Self {
        Self {
            offset: NexusAttribute::new_with_default("offset"),
        }
    }
}

struct EventTimeZeroMessage<'a> {
    frame_assembled_event_list: &'a FrameAssembledEventListMessage<'a>,
    has_offset: bool,
}

impl<'a> EventTimeZeroMessage<'a> {
    fn get_timestamp(&self) -> Result<DateTime<Utc>, NexusPushError> {
        (*self
            .frame_assembled_event_list
            .metadata()
            .timestamp()
            .ok_or(NexusMissingEventlistError::Timestamp)
            .map_err(NexusMissingError::Eventlist)?)
        .try_into()
        .map_err(NexusConversionError::GpsTimeConversion)
        .map_err(NexusPushError::Conversion)
    }
}

fn datetime_diff_to_u64(
    timestamp: DateTime<Utc>,
    offset: DateTime<Utc>,
) -> Result<u64, NexusConversionError> {
    (timestamp - offset)
        .num_nanoseconds()
        .ok_or(NexusConversionError::NanosecondError(timestamp - offset))?
        .try_into()
        .map_err(NexusConversionError::TimeDeltaNegative)
}

impl<'a> NexusHandleMessage<EventTimeZeroMessage<'a>, Dataset, u64> for EventTimeZero {
    fn handle_message(
        &mut self,
        message: &EventTimeZeroMessage<'a>,
        _dataset: &Dataset,
    ) -> Result<u64, NexusPushError> {
        let timestamp: DateTime<Utc> = message.get_timestamp()?;

        let time_zero = {
            if message.has_offset {
                let offset = DateTime::<Utc>::from_str(self.offset.read_scalar(_dataset)?.as_str())
                    .map_err(NexusConversionError::ChronoParse)?;

                datetime_diff_to_u64(timestamp, offset)?
            } else {
                self.offset
                    .write_string(_dataset, &timestamp.to_rfc3339())?;

                u64::default()
            }
        };
        Ok(time_zero)
    }
}

/*
    Group: Data
*/
pub(super) struct Data {
    /// list of channels attributed to each detection event
    event_id: NexusDatasetResize<u32>,
    /// list of indices of the first detection event in a frame
    event_index: NexusDatasetResize<usize>,
    /// list of times attributed to each detection event relative to the start of the frame
    event_time_offset: NexusDatasetResize<u32, EventTimeOffset>,
    /// list of times each frame
    event_time_zero: NexusDatasetResize<u64, EventTimeZero>,
    /// list of periods of each frame
    event_period_number: NexusDatasetResize<u64>,
    /// list of intensities attributed to each detection event
    event_pulse_height: NexusDatasetResize<f64>,
}

impl NexusGroupDef for Data {
    const CLASS_NAME: &'static str = nexus_class::EVENT_DATA;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            event_id: NexusDataset::new_appendable_with_default(
                "event_id",
                settings.eventlist_chunk_size,
            ),
            event_index: NexusDataset::new_appendable_with_default(
                "event_index",
                settings.framelist_chunk_size,
            ),
            event_time_offset: NexusDataset::new_appendable_with_default(
                "event_time_offset",
                settings.eventlist_chunk_size,
            ),
            event_time_zero: NexusDataset::new_appendable_with_default(
                "event_time_zero",
                settings.framelist_chunk_size,
            ),
            event_period_number: NexusDataset::new_appendable_with_default(
                "event_period_number",
                settings.framelist_chunk_size,
            ),
            event_pulse_height: NexusDataset::new_appendable_with_default(
                "event_pulse_height",
                settings.eventlist_chunk_size,
            ),
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>> for Data {
    fn handle_message(
        &mut self,
        _message: &RunStart<'a>,
        _group: &Group,
    ) -> Result<(), NexusPushError> {
        //let timestamp = DateTime::<Utc>::from_timestamp_millis(i64::try_from(message.start_time())?).ok_or(anyhow::anyhow!("Millisecond error"))?;
        //self.event_time_zero.attributes(|attributes|Ok(attributes.offset.write_scalar(timestamp.to_rfc3339().parse()?)?))?;
        Ok(())
    }
}

impl<'a> NexusHandleMessage<FrameAssembledEventListMessage<'a>> for Data {
    fn handle_message(
        &mut self,
        message: &FrameAssembledEventListMessage<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        // Here is where we extend the datasets

        //  event_id
        let current_index = self.event_id.get_size(parent)?;
        self.event_id.append(
            parent,
            &message
                .channel()
                .ok_or(NexusMissingEventlistError::Channels)
                .map_err(NexusMissingError::Eventlist)?
                .iter()
                .collect::<Vec<_>>(),
        )?;

        //  event_time_offset
        self.event_time_offset.append(
            parent,
            &message
                .time()
                .ok_or(NexusMissingEventlistError::Times)
                .map_err(NexusMissingError::Eventlist)?
                .iter()
                .collect::<Vec<_>>(),
        )?;
        self.event_time_offset.close_hdf5();

        //  event_pulse_height
        self.event_pulse_height.append(
            parent,
            &message
                .voltage()
                .ok_or(NexusMissingEventlistError::Voltages)
                .map_err(NexusMissingError::Eventlist)?
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
        let event_time_zero_attributes_message = EventTimeZeroMessage {
            frame_assembled_event_list: message,
            has_offset: current_index != 0,
        };

        let time_zero = self
            .event_time_zero
            .push_message(&event_time_zero_attributes_message, parent)?;

        self.event_time_zero.append(parent, &[time_zero])?;
        Ok(())
    }
}
