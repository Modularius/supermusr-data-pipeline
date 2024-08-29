use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::schematic::{
    elements::{
        attribute::{NexusAttribute, RcNexusAttributeVar},
        dataset::{NexusDataset, NxDataset, AttributeRegister},traits::Buildable,
        group::{NexusGroup, NxGroup, NxPushMessage, RcGroupContentRegister, RcNexusGroup},
    },
    groups::log::Log,
    nexus_class, H5String,
};

#[derive(Clone)]
struct FramesRequestedAttributes {
    frame_type: RcNexusAttributeVar<H5String>,
}

impl NxDataset for FramesRequestedAttributes {
    fn new(attribute_register: AttributeRegister) -> Self {
        Self {
            frame_type: NexusAttribute::begin().finish("frame_type", attribute_register.clone()),
        }
    }
}

#[derive(Clone)]
struct LabelsAttributes {
    separator: RcNexusAttributeVar<H5String>,
}

impl NxDataset for LabelsAttributes {
    fn new(attribute_register: AttributeRegister) -> Self {
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
            number: NexusDataset::begin("number").finish(&dataset_register),
            period_types: NexusDataset::begin("type").finish(&dataset_register),
            frames_requested: NexusDataset::begin("frames_requested").finish(&dataset_register),
            output: NexusDataset::begin("output").finish(&dataset_register),
            labels: NexusDataset::begin("labels").finish(&dataset_register),
            raw_frames: NexusDataset::begin("raw_frames").finish(&dataset_register),
            good_frames: NexusDataset::begin("good_frames").finish(&dataset_register),
            sequences: NexusDataset::begin("sequences").finish(&dataset_register),
            counts: NexusGroup::new("counts", &dataset_register),
        }
    }
}

impl<'a> NxPushMessage<RunStart<'a>> for Periods {
    type MessageType = RunStart<'a>;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        Ok(())
    }
}
