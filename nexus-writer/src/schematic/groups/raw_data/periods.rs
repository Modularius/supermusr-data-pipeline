use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::schematic::{
    elements::{
        attribute::{NexusAttribute, RcNexusAttributeVar},
        dataset::{Buildable, NexusDataset, NxContainerAttributes, RcAttributeRegister},
        group::{NexusGroup, NxGroup, NxPushMessage, RcGroupContentRegister, RcNexusGroup},
    },
    groups::log::Log,
    nexus_class, H5String,
};

#[derive(Clone)]
struct FramesRequestedAttributes {
    frame_type: RcNexusAttributeVar<H5String>,
}

impl NxContainerAttributes for FramesRequestedAttributes {
    fn new(attribute_register: RcAttributeRegister) -> Self {
        Self {
            frame_type: NexusAttribute::begin().finish("frame_type", attribute_register.clone()),
        }
    }
}

#[derive(Clone)]
struct LabelsAttributes {
    separator: RcNexusAttributeVar<H5String>,
}

impl NxContainerAttributes for LabelsAttributes {
    fn new(attribute_register: RcAttributeRegister) -> Self {
        Self {
            separator: NexusAttribute::begin().finish("separator", attribute_register.clone()),
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
    counts: RcNexusGroup<Log>,
}

impl NxGroup for Periods {
    const CLASS_NAME: &'static str = nexus_class::PERIOD;

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            number: NexusDataset::begin().finish("number", dataset_register.clone()),
            period_types: NexusDataset::begin().finish("type", dataset_register.clone()),
            frames_requested: NexusDataset::begin()
                .finish("frames_requested", dataset_register.clone()),
            output: NexusDataset::begin().finish("output", dataset_register.clone()),
            labels: NexusDataset::begin().finish("labels", dataset_register.clone()),
            raw_frames: NexusDataset::begin().finish("raw_frames", dataset_register.clone()),
            good_frames: NexusDataset::begin().finish("good_frames", dataset_register.clone()),
            sequences: NexusDataset::begin().finish("sequences", dataset_register.clone()),
            counts: NexusGroup::new("counts", Some(dataset_register)),
        }
    }
}

impl<'a> NxPushMessage<RunStart<'a>> for Periods {
    type MessageType = RunStart<'a>;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        Ok(())
    }
}
