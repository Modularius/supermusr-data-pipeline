use hdf5::Group;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::{
    nexus::NexusSettings,
    schematic::{
        elements::{
            attribute::NexusAttribute, dataset::NexusDataset, group::NexusGroup, NexusBuildable,
            NexusDatasetDef, NexusError, NexusGroupDef, NexusHandleMessage,
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
    number: NexusDataset<u32>,
    period_types: NexusDataset<u32>,
    frames_requested: NexusDataset<u32, FramesRequestedAttributes>,
    output: NexusDataset<u32>,
    labels: NexusDataset<H5String, LabelsAttributes>,
    raw_frames: NexusDataset<u32>,
    good_frames: NexusDataset<u32>,
    sequences: NexusDataset<u32>,
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
    ) -> Result<(), NexusError> {
        Ok(())
    }
}
