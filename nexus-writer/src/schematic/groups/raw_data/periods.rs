use hdf5::types::{TypeDescriptor, VarLenAscii};
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::schematic::{
    elements::{
        attribute::NexusAttribute,
        dataset::{MustEnterAttributes, NexusDataset, RcNexusDatasetVar},
        group::{NexusGroup, NxGroup, NxPushMessage, RcDatasetRegister},
    },
    groups::log::Log,
};

pub(super) struct Periods {
    number: RcNexusDatasetVar<u32>,
    period_types: RcNexusDatasetVar<u32>,
    frames_requested: RcNexusDatasetVar<u32, MustEnterAttributes<1>>,
    output: RcNexusDatasetVar<u32>,
    labels: RcNexusDatasetVar<VarLenAscii, MustEnterAttributes<1>>,
    raw_frames: RcNexusDatasetVar<u32>,
    good_frames: RcNexusDatasetVar<u32>,
    sequences: RcNexusDatasetVar<u32>,
    counts: NexusGroup<Log>,
}

impl NxGroup for Periods {
    const CLASS_NAME: &'static str = "NXperiod";

    fn new(dataset_register : RcDatasetRegister) -> Self {
        Self {
            number: NexusDataset::begin().finish("number", dataset_register.clone()),
            period_types: NexusDataset::begin().finish("type", dataset_register.clone()),
            frames_requested: NexusDataset::begin()
                .attributes([NexusAttribute::new(
                    "frame_type",
                    TypeDescriptor::VarLenAscii,
                )])
                .finish("frames_requested", dataset_register.clone()),
            output: NexusDataset::begin().finish("output", dataset_register.clone()),
            labels: NexusDataset::begin()
                .attributes([NexusAttribute::new(
                    "separator",
                    TypeDescriptor::VarLenAscii,
                )])
                .finish("labels", dataset_register.clone()),
            raw_frames: NexusDataset::begin().finish("raw_frames", dataset_register.clone()),
            good_frames: NexusDataset::begin().finish("good_frames", dataset_register.clone()),
            sequences: NexusDataset::begin().finish("sequences", dataset_register.clone()),
            counts: NexusGroup::new("counts"),
        }
    }
}

impl<'a> NxPushMessage<RunStart<'a>> for Periods {
    type MessageType = RunStart<'a>;
    
    fn push_message(&mut self, message: &Self::MessageType) {
        todo!()
    }
}