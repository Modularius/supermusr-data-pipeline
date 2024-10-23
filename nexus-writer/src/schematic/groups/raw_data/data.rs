use chrono::{DateTime, Utc};
use hdf5::{Dataset, Group};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_pl72_run_start_generated::RunStart,
};
use tracing::info;

use crate::{
    elements::{
        attribute::{NexusAttribute, NexusAttributeMut},
        dataset::{NexusDataset, NexusDatasetResize, NexusDatasetResizeMut},
        traits::{
            NexusAppendableDataHolder, NexusDataHolderScalarMutable, NexusDataHolderStringMutable,
            NexusDataHolderVectorMutable, NexusDataHolderWithSize, NexusDatasetDef,
            NexusDatasetDefUnitsOnly, NexusGroupDef, NexusH5CreatableDataHolder,
            NexusHandleMessage, NexusPushMessage,
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

#[derive(Default)]
struct EventTimeZeroResult {
    time_zero: u64,
    offset_diff: Option<u64>,
}

impl EventTimeZeroResult {
    fn new(offset_diff: i64) -> Self {
        // if new datetime is before the current offset, then previous event_time_zero entries should be ammended.
        if offset_diff < 0 {
            info!("offset_diff = {offset_diff} < 0");
            Self {
                time_zero: u64::default(),
                // all existing event_time_zero entries are `value - old_offset`,
                // and as `offset_diff = new_offset - old_offset`,
                // the updated event_time_zero entries should be
                // `value - new_offset = value - old_offset - offset_diff`.
                // So existing entries should add `-offset_diff` to themselves.
                offset_diff: Some(
                    (-offset_diff)
                        .try_into()
                        .expect("-offset_delta should be non-negative"),
                ),
            }
        } else {
            info!("offset_diff = {offset_diff} >= 0");
            // Otherwise only the current one should be ammended
            Self {
                time_zero: offset_diff
                    .try_into()
                    .expect("offset_delta should be non-negative"),
                offset_diff: None,
            }
        }
    }
}

impl<'a> NexusHandleMessage<DateTime<Utc>, Dataset, EventTimeZeroResult> for EventTimeZero {
    fn handle_message(
        &mut self,
        timestamp: &DateTime<Utc>,
        dataset: &Dataset,
    ) -> Result<EventTimeZeroResult, NexusPushError> {
        let offset_string = self.offset.read_scalar(dataset)?;

        if offset_string.is_empty() {
            self.offset.write_string(dataset, &timestamp.to_rfc3339())?;
            info!("No offset found, {0} written", timestamp.to_rfc3339());

            Ok(EventTimeZeroResult::default())
        } else {
            info!("Found offset {offset_string}");
            // Get current offset datetime
            let offset: DateTime<Utc> = offset_string
                .parse()
                .map_err(NexusConversionError::ChronoParse)?;

            // get integer nanosecond difference
            let offset_diff = (*timestamp - offset)
                .num_nanoseconds()
                .ok_or_else(|| NexusConversionError::NanosecondError(*timestamp - offset))?;

            if offset_diff < 0 {
                self.offset.write_string(dataset, &timestamp.to_rfc3339())?;
            }

            Ok(EventTimeZeroResult::new(offset_diff))
        }
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
    event_time_zero: NexusDatasetResizeMut<u64, EventTimeZero>,
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

        // event_time_zero
        let time_zero: DateTime<Utc> =
            (*message
                .metadata()
                .timestamp()
                .ok_or(NexusMissingError::Eventlist(
                    NexusMissingEventlistError::Timestamp,
                ))?)
            .try_into()
            .map_err(NexusConversionError::GpsTimeConversion)?;

        let event_time_zero_result = self.event_time_zero.push_message(&time_zero, parent)?;

        //  If offset_diff is set, then add it to the existing event_time_zero entries
        if let Some(offset_diff) = event_time_zero_result.offset_diff {
            self.event_time_zero
                .mutate_all_in_place(parent, |value| *value + offset_diff)?;
        }
        self.event_time_zero
            .append(parent, &[event_time_zero_result.time_zero])?;
        Ok(())
    }
}
