use hdf5::Group;
use supermusr_streaming_types::{aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage, ecs_pl72_run_start_generated::RunStart};

use crate::{
    elements::{
        attribute::{NexusAttribute, NexusAttributeMut},
        dataset::{NexusDataset, NexusDatasetMut, NexusDatasetResize},
        group::NexusGroup,
        traits::{
            NexusAppendableDataHolder, NexusDataHolderScalarMutable, NexusDatasetDef, NexusGroupDef, NexusHandleMessage
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
    number: NexusDatasetMut<u32>,
    period_types: NexusDatasetResize<u32>,
    frames_requested: NexusDatasetResize<u32, FramesRequestedAttributes>,
    output: NexusDatasetResize<u32>,
    labels: NexusDatasetMut<H5String, LabelsAttributes>,
    raw_frames: NexusDatasetResize<u32>,
    good_frames: NexusDatasetResize<u32>,
    sequences: NexusDatasetResize<u32>,
    counts: NexusGroup<Log>,
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
            counts: NexusGroup::new("counts", settings),
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
        let p = message.metadata().period_number();

        Ok(())
    }
}
