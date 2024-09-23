use hdf5::Group;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::{
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{
        elements::{
            attribute::NexusAttribute,
            dataset::{NexusDataset, NexusDatasetMut},
            group::NexusGroup,
            traits::{NexusBuildable, NexusDatasetDef, NexusGroupDef, NexusHandleMessage},
        },
        groups::log::Log,
        nexus_class, H5String,
    },
};

#[derive(Clone)]
struct FramesRequestedAttributes {
    frame_type: NexusAttribute<H5String>,
}

impl NexusDatasetDef for FramesRequestedAttributes {
    fn new() -> Self {
        Self {
            frame_type: NexusAttribute::begin("frame_type").finish_with_auto_default(),
        }
    }
}

#[derive(Clone)]
struct LabelsAttributes {
    separator: NexusAttribute<H5String>,
}

impl NexusDatasetDef for LabelsAttributes {
    fn new() -> Self {
        Self {
            separator: NexusAttribute::begin("separator").finish_with_auto_default(),
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
            number: NexusDataset::begin("number").finish_with_auto_default(),
            period_types: NexusDataset::begin("type").finish_with_auto_default(),
            frames_requested: NexusDataset::begin("frames_requested").finish_with_auto_default(),
            output: NexusDataset::begin("output").finish_with_auto_default(),
            labels: NexusDataset::begin("labels").finish_with_auto_default(),
            raw_frames: NexusDataset::begin("raw_frames").finish_with_auto_default(),
            good_frames: NexusDataset::begin("good_frames").finish_with_auto_default(),
            sequences: NexusDataset::begin("sequences").finish_with_auto_default(),
            counts: NexusGroup::new("counts", settings),
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>> for Periods {
    fn handle_message(
        &mut self,
        message: &RunStart<'a>,
        location: &Group,
    ) -> Result<(), NexusPushError> {
        Ok(())
    }
}
