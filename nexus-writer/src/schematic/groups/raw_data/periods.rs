use hdf5::Group;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::{
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{
        elements::{
            attribute::{NexusAttribute, NexusAttributeMut},
            dataset::{NexusDataset, NexusDatasetMut},
            group::NexusGroup,
            traits::{NexusDataHolderScalarMutable, NexusDatasetDef, NexusGroupDef, NexusHandleMessage},
        },
        groups::log::Log,
        nexus_class, H5String,
    },
};

#[derive(Clone)]
struct FramesRequestedAttributes {
    frame_type: NexusAttributeMut<H5String>,
}

impl NexusDatasetDef for FramesRequestedAttributes {
    fn new() -> Self {
        Self {
            frame_type: NexusAttribute::new_with_auto_default("frame_type"),
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
            separator: NexusAttribute::new_with_auto_default("separator"),
        }
    }
}

pub(super) struct Periods {
    number: NexusDatasetMut<u32>,
    period_types: NexusDatasetMut<u32>,
    frames_requested: NexusDatasetMut<u32, FramesRequestedAttributes>,
    output: NexusDatasetMut<u32>,
    labels: NexusDatasetMut<H5String, LabelsAttributes>,
    raw_frames: NexusDatasetMut<u32>,
    good_frames: NexusDatasetMut<u32>,
    sequences: NexusDatasetMut<u32>,
    counts: NexusGroup<Log>,
}

impl NexusGroupDef for Periods {
    const CLASS_NAME: &'static str = nexus_class::PERIOD;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            number: NexusDataset::new_with_auto_default("number"),
            period_types: NexusDataset::new_with_auto_default("type"),
            frames_requested: NexusDataset::new_with_auto_default("frames_requested"),
            output: NexusDataset::new_with_auto_default("output"),
            labels: NexusDataset::new_with_auto_default("labels"),
            raw_frames: NexusDataset::new_with_auto_default("raw_frames"),
            good_frames: NexusDataset::new_with_auto_default("good_frames"),
            sequences: NexusDataset::new_with_auto_default("sequences"),
            counts: NexusGroup::new("counts", settings),
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>> for Periods {
    fn handle_message(
        &mut self,
        _message: &RunStart<'a>,
        _location: &Group,
    ) -> Result<(), NexusPushError> {
        Ok(())
    }
}
