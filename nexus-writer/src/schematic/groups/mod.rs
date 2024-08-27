use hdf5::types::VarLenAscii;
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm,
    ecs_f144_logdata_generated::f144_LogData, ecs_pl72_run_start_generated::RunStart,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};

use crate::schematic::elements::{
    attribute::NexusAttribute,
    group::{NexusGroup, NxGroup},
};

use super::elements::{
    attribute::{RcNexusAttributeFixed, RcNexusAttributeVar},
    dataset::{NxContainerAttributes, RcAttributeRegister},
    group::{NxPushMessage, RcGroupContentRegister, RcNexusGroup},
};

pub(super) mod log;
pub(crate) mod raw_data;

struct RawData1Attributes {
    file_name: RcNexusAttributeVar<VarLenAscii>,
    file_time: RcNexusAttributeVar<VarLenAscii>,
    initial_file_format: RcNexusAttributeFixed<VarLenAscii>,
    nexus_version: RcNexusAttributeFixed<VarLenAscii>,
    hdf_version: RcNexusAttributeFixed<VarLenAscii>,
    hdf5_version: RcNexusAttributeFixed<VarLenAscii>,
    xml_version: RcNexusAttributeFixed<VarLenAscii>,
    creator: RcNexusAttributeFixed<VarLenAscii>,
}

impl NxContainerAttributes for RawData1Attributes {
    fn new(attribute_register: RcAttributeRegister) -> Self {
        Self {
            file_name: NexusAttribute::begin().finish("file_name", attribute_register.clone()),
            file_time: NexusAttribute::begin().finish("file_time", attribute_register.clone()),
            initial_file_format: NexusAttribute::begin()
                .fixed_value(VarLenAscii::from_ascii("TODO").expect(""))
                .finish("initial_file_format", attribute_register.clone()),
            nexus_version: NexusAttribute::begin()
                .fixed_value(VarLenAscii::from_ascii("TODO").expect(""))
                .finish("nexus_version", attribute_register.clone()),
            hdf_version: NexusAttribute::begin()
                .fixed_value(VarLenAscii::from_ascii("TODO").expect(""))
                .finish("hdf_version", attribute_register.clone()),
            hdf5_version: NexusAttribute::begin()
                .fixed_value(VarLenAscii::from_ascii("TODO").expect(""))
                .finish("hdf5_version", attribute_register.clone()),
            xml_version: NexusAttribute::begin()
                .fixed_value(VarLenAscii::from_ascii("TODO").expect(""))
                .finish("xml_version", attribute_register.clone()),
            creator: NexusAttribute::begin()
                .fixed_value(VarLenAscii::from_ascii("TODO").expect(""))
                .finish("creator", attribute_register.clone()),
        }
    }
}

pub(crate) struct NXRoot {
    raw_data_1: RcNexusGroup<raw_data::RawData>,
}

impl NxGroup for NXRoot {
    const CLASS_NAME: &'static str = "NXroot";

    fn new(database_register: RcGroupContentRegister) -> Self {
        Self {
            raw_data_1: NexusGroup::new("raw_data_1", Some(database_register)),
        }
    }
}

/*
pub(crate) mod sample;
pub(crate) mod geometry;
pub(crate) mod environment;
pub(crate) mod log;
pub(crate) mod selog;
pub(crate) mod user;
*/

impl<'a> NxPushMessage<FrameAssembledEventListMessage<'a>> for NXRoot {
    type MessageType = FrameAssembledEventListMessage<'a>;

    fn push_message(&mut self, message: &Self::MessageType) {
        self.raw_data_1.push_message(message)
    }
}

impl<'a> NxPushMessage<RunStart<'a>> for NXRoot {
    type MessageType = RunStart<'a>;

    fn push_message(&mut self, message: &Self::MessageType) {
        self.raw_data_1.push_message(message)
    }
}
impl<'a> NxPushMessage<RunStop<'a>> for NXRoot {
    type MessageType = RunStop<'a>;

    fn push_message(&mut self, message: &Self::MessageType) {
        self.raw_data_1.push_message(message)
    }
}

impl<'a> NxPushMessage<Alarm<'a>> for NXRoot {
    type MessageType = Alarm<'a>;

    fn push_message(&mut self, message: &Self::MessageType) {
        self.raw_data_1.push_message(message)
    }
}

impl<'a> NxPushMessage<se00_SampleEnvironmentData<'a>> for NXRoot {
    type MessageType = Alarm<'a>;

    fn push_message(&mut self, message: &Self::MessageType) {
        self.raw_data_1.push_message(message)
    }
}

impl<'a> NxPushMessage<f144_LogData<'a>> for NXRoot {
    type MessageType = Alarm<'a>;

    fn push_message(&mut self, message: &Self::MessageType) {
        self.raw_data_1.push_message(message)
    }
}
