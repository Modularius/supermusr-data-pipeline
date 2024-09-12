use hdf5::{Group, Location};
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::{
    nexus::NexusSettings,
    schematic::{
        elements::{
            attribute::NexusAttribute, dataset::NexusDataset, group::NexusGroup, NexusBuildable,
            NexusBuilderFinished, NexusDatasetDef, NexusError, NexusGroupDef, NexusHandleMessage,
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
            frame_type: NexusAttribute::begin("frame_type")
                .default_value(Default::default())
                .finish(),
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
            separator: NexusAttribute::begin("separator")
                .default_value(Default::default())
                .finish(),
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
            number: NexusDataset::begin("number")
                .default_value(Default::default())
                .finish(),
            period_types: NexusDataset::begin("type")
                .default_value(Default::default())
                .finish(),
            frames_requested: NexusDataset::begin("frames_requested")
                .default_value(Default::default())
                .finish(),
            output: NexusDataset::begin("output")
                .default_value(Default::default())
                .finish(),
            labels: NexusDataset::begin("labels")
                .default_value(Default::default())
                .finish(),
            raw_frames: NexusDataset::begin("raw_frames")
                .default_value(Default::default())
                .finish(),
            good_frames: NexusDataset::begin("good_frames")
                .default_value(Default::default())
                .finish(),
            sequences: NexusDataset::begin("sequences")
                .default_value(Default::default())
                .finish(),
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
