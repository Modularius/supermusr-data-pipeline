use std::rc::Rc;

use hdf5::{
    types::{IntSize, TypeDescriptor, VarLenAscii},
    Group,
};
use supermusr_streaming_types::aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage;

use crate::schematic::{elements::{
    attribute::{NexusAttribute, NexusUnits, RcNexusAttributeFixed, RcNexusAttributeVar},
    dataset::{NexusDataset, NxContainerAttributes, RcAttributeRegister, RcNexusDatasetResize, RcNexusDatasetVar},
    group::{NxGroup, NxPushMessage, RcGroupContentRegister},
}, nexus_class};

#[derive(Clone)]
struct EventTimeOffsetAttributes {}

impl NxContainerAttributes for EventTimeOffsetAttributes {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Nanoseconds);

    fn new(_attribute_register: RcAttributeRegister) -> Self {
        Self {}
    }
}

#[derive(Clone)]
struct EventTimeZeroAttributes {
    offset: RcNexusAttributeVar<VarLenAscii>,
}

impl NxContainerAttributes for EventTimeZeroAttributes {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Nanoseconds);

    fn new(attribute_register: RcAttributeRegister) -> Self {
        Self {
            offset: NexusAttribute::begin().finish("offset", attribute_register.clone()),
        }
    }
}

pub(super) struct Data {
    event_id: RcNexusDatasetResize<u32>,
    event_index: RcNexusDatasetResize<u32>,
    event_time_offset: RcNexusDatasetResize<u32, EventTimeOffsetAttributes>,
    event_time_zero: RcNexusDatasetResize<u64, EventTimeZeroAttributes>,
    event_period_number: RcNexusDatasetResize<u64>,
    event_pulse_height: RcNexusDatasetResize<f64>,
}

impl NxGroup for Data {
    const CLASS_NAME: &'static str = nexus_class::EVENT_DATA;

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            event_id: NexusDataset::begin().resizable(0, 128).finish("event_id", dataset_register.clone()),
            event_index: NexusDataset::begin().resizable(0, 128).finish("event_index", dataset_register.clone()),
            event_time_offset: NexusDataset::begin().resizable(0, 128)
                .finish("event_time_offset", dataset_register.clone()),
            event_time_zero: NexusDataset::begin().resizable(0, 128)
                .finish("event_time_zero", dataset_register.clone()),
            event_period_number: NexusDataset::begin().resizable(0, 128)
                .finish("event_period_number", dataset_register.clone()),
            event_pulse_height: NexusDataset::begin().resizable(0, 128)
                .finish("event_pulse_height", dataset_register.clone()),
        }
    }
}

impl<'a> NxPushMessage<FrameAssembledEventListMessage<'a>> for Data {
    type MessageType = FrameAssembledEventListMessage<'a>;

    fn push_message(&self, message: &Self::MessageType) {
        // Here is where we extend the datasets
    }
}
