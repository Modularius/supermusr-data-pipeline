use std::rc::Rc;

use hdf5::{
    types::{IntSize, TypeDescriptor, VarLenAscii},
    Group,
};
use supermusr_streaming_types::aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage;

use crate::schematic::elements::{
    attribute::{NexusAttribute, NexusUnits},
    dataset::{MustEnterAttributes, NexusDataset, RcNexusDatasetVar},
    group::{NxGroup, NxPushMessage, RcDatasetRegister},
};

pub(super) struct Data {
    event_id: RcNexusDatasetVar<u32>,
    event_index: RcNexusDatasetVar<u32>,
    event_time_offset: RcNexusDatasetVar<u32, MustEnterAttributes<1>>,
    event_time_zero: RcNexusDatasetVar<u64, MustEnterAttributes<2>>,
    event_period_number: RcNexusDatasetVar<u64>,
    event_pulse_height: RcNexusDatasetVar<f64>,
}

impl NxGroup for Data {
    const CLASS_NAME: &'static str = "NXperiod";

    fn new(dataset_register : RcDatasetRegister) -> Self {
        Self {
            event_id: NexusDataset::begin().finish("event_id", dataset_register.clone()),
            event_index: NexusDataset::begin().finish("event_index", dataset_register.clone()),
            event_time_offset: NexusDataset::begin()
                .attributes([NexusAttribute::units(NexusUnits::Nanoseconds)])
                .finish("event_time_offset", dataset_register.clone()),
            event_time_zero: NexusDataset::begin()
                .attributes([
                    NexusAttribute::units(NexusUnits::Nanoseconds),
                    NexusAttribute::new("Start", TypeDescriptor::Integer(IntSize::U4)),
                ])
                .finish("event_time_zero", dataset_register.clone()),
            event_period_number: NexusDataset::begin().finish("event_period_number", dataset_register.clone()),
            event_pulse_height: NexusDataset::begin().finish("event_pulse_height", dataset_register.clone()),
        }
    }
}


impl<'a> NxPushMessage<FrameAssembledEventListMessage<'a>> for Data {
    type MessageType = FrameAssembledEventListMessage<'a>;

    fn push_message(&mut self, message: &Self::MessageType) {
        // Here is where we extend the datasets
    }
}