use hdf5::{Dataset, Group};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_pl72_run_start_generated::RunStart,
};

use crate::{
    elements::{
        attribute::{NexusAttribute, NexusAttributeMut},
        dataset::{NexusDataset, NexusDatasetResize, NexusDatasetResizeMut},
        traits::{
            NexusAppendableDataHolder, NexusDataHolderScalarMutable, NexusDataHolderStringMutable,
            NexusDataHolderWithSize, NexusDatasetDef, NexusDatasetDefUnitsOnly, NexusGroupDef,
            NexusH5CreatableDataHolder, NexusHandleMessage, NexusPushMessage,
        },
        NexusUnits,
    },
    error::{NexusConversionError, NexusMissingError, NexusMissingEventlistError, NexusPushError},
    nexus::{FrameParameters, NexusSettings},
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

impl NexusHandleMessage<Vec<FrameParameters>, Dataset, i64> for EventTimeZero {
    fn handle_message(
        &mut self,
        frame_parameters: &Vec<FrameParameters>,
        dataset: &Dataset,
    ) -> Result<i64, NexusPushError> {
        let timestamp = frame_parameters
            .iter()
            .map(|fp| fp.datetime)
            .min()
            .expect("Frames should have nonzero length");

        self.offset.write_string(dataset, &timestamp.to_rfc3339())?;
        Ok(timestamp
            .timestamp_nanos_opt()
            .ok_or(NexusConversionError::NanosecondError(timestamp))?)
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

        Ok(())
    }
}

impl NexusHandleMessage<Vec<FrameParameters>> for Data {
    fn handle_message(
        &mut self,
        message: &Vec<FrameParameters>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        let offset = self.event_time_zero.push_message(message, parent)?;

        for fp in message {
            let time_zero = fp
                .datetime
                .timestamp_nanos_opt()
                .ok_or(NexusConversionError::NanosecondError(fp.datetime))?;

            self.event_time_zero.append(
                parent,
                &[(time_zero - offset)
                    .try_into()
                    .expect("Should be nonnegative")],
            )?;
        }
        Ok(())
    }
}
