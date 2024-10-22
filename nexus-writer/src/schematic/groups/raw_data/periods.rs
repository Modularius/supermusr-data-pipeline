use hdf5::Group;
use supermusr_streaming_types::{aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage, ecs_pl72_run_start_generated::RunStart};

use crate::{
    elements::{
        attribute::{NexusAttribute, NexusAttributeMut},
        dataset::{NexusDataset, NexusDatasetMut, NexusDatasetResize, NexusDatasetResizeMut},
        group::NexusGroup,
        traits::{
            NexusAppendableDataHolder, NexusDataHolderScalarMutable, NexusDataHolderVectorMutable, NexusDataHolderWithSize, NexusDatasetDef, NexusGroupDef, NexusHandleMessage, NexusSearchableDataHolder
        },
    },
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{groups::log::Log, nexus_class, H5String},
};

#[derive(Clone)]
struct FramesRequestedAttributes {
    frame_type: NexusAttributeMut<H5String>,
}

impl NexusDatasetDef for FramesRequestedAttributes {
    fn new() -> Self {
        Self {
            frame_type: NexusAttribute::new_with_default("frame_type"),
        }
    }
}

#[derive(Clone)]
struct LabelsAttributes {
    separator: NexusAttributeMut<H5String>,
}

impl NexusDatasetDef for LabelsAttributes {
    fn new() -> Self {
        Self {
            separator: NexusAttribute::new_with_default("separator"),
        }
    }
}

pub(super) struct Periods {
    /// number of periods used
    number: NexusDatasetMut<u32>,
    /// unction of period: ‘1’ – DAQ, ‘2’ – DWELL
    period_types: NexusDatasetResizeMut<u32>,
    /// frames collected in each period before switching, ‘0’ for unlimited frames
    frames_requested: NexusDatasetResizeMut<u32, FramesRequestedAttributes>,
    /// output bit pattern on period card. If not known, write '0' ... `np` - 1 into array
    output: NexusDatasetResize<u64>,
    /// list of period names, separated by character given as attribute.May use a 2D array of NX_CHAR - TBC
    labels: NexusDatasetMut<H5String, LabelsAttributes>,
    /// raw frames collected for each period
    raw_frames: NexusDatasetResizeMut<u32>,
    /// good frames collected for each period
    good_frames:NexusDatasetResizeMut<u32>,
    /// number of times data collection took place in each period
    sequences: NexusDatasetResizeMut<u32>,
    /// counts collected in each periods
    _counts: NexusGroup<Log>,
}

impl NexusGroupDef for Periods {
    const CLASS_NAME: &'static str = nexus_class::PERIOD;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            number: NexusDataset::new_with_default("number"),
            period_types: NexusDataset::new_appendable_with_default("type", settings.periodlist_chunk_size),
            frames_requested: NexusDataset::new_appendable_with_default("frames_requested", settings.periodlist_chunk_size),
            output: NexusDataset::new_appendable_with_default("output", settings.periodlist_chunk_size),
            labels: NexusDataset::new_with_default("labels"),
            raw_frames: NexusDataset::new_appendable_with_default("raw_frames", settings.periodlist_chunk_size),
            good_frames: NexusDataset::new_appendable_with_default("good_frames", settings.periodlist_chunk_size),
            sequences: NexusDataset::new_appendable_with_default("sequences", settings.periodlist_chunk_size),
            _counts: NexusGroup::new("counts", settings),
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>> for Periods {
    fn handle_message(
        &mut self,
        message: &RunStart<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        Ok(())
    }
}

impl<'a> NexusHandleMessage<FrameAssembledEventListMessage<'a>> for Periods {
    fn handle_message(
        &mut self,
        message: &FrameAssembledEventListMessage<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        let period_number = message.metadata().period_number();
        if let Some(index) = self.output.find(parent, period_number)? {
            self.output.append(parent, &[period_number])?;
            self.raw_frames.mutate_in_place(parent, index, |x|x + 1)?;
            if message.metadata().veto_flags() == 0 {
                self.good_frames.mutate_in_place(parent, index, |x|x + 1)?;
            }
            self.frames_requested.mutate_in_place(parent, index, |x|x + 1)?;
        } else {
            self.output.append(parent, &[period_number])?;
            self.raw_frames.append(parent, &[1])?;
            self.good_frames.append(parent, &[if message.metadata().veto_flags() == 0 {1} else {0}])?;
            self.period_types.append(parent, &[0])?;
            self.frames_requested.append(parent, &[0])?;
            self.number.mutate(parent, |x|x + 1)?;
        };
        
        Ok(())
    }
}
