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

use super::{
    elements::{
        attribute::{RcNexusAttributeFixed, RcNexusAttributeVar},
        dataset::{NxContainerAttributes, RcAttributeRegister},
        group::{NxPushMessage, NxPushMessageMut, RcGroupContentRegister, RcNexusGroup},
    },
    nexus_class, H5String,
};

pub(super) mod log;
pub(crate) mod raw_data;

struct RawData1Attributes {
    file_name: RcNexusAttributeVar<H5String>,
    file_time: RcNexusAttributeVar<H5String>,
    initial_file_format: RcNexusAttributeFixed<H5String>,
    nexus_version: RcNexusAttributeFixed<H5String>,
    hdf_version: RcNexusAttributeFixed<H5String>,
    hdf5_version: RcNexusAttributeFixed<H5String>,
    xml_version: RcNexusAttributeFixed<H5String>,
    creator: RcNexusAttributeFixed<H5String>,
}

impl NxContainerAttributes for RawData1Attributes {
    fn new(attribute_register: RcAttributeRegister) -> Self {
        Self {
            file_name: NexusAttribute::begin().finish("file_name", attribute_register.clone()),
            file_time: NexusAttribute::begin().finish("file_time", attribute_register.clone()),
            initial_file_format: NexusAttribute::begin()
                .fixed_value("TODO".parse().expect(""))
                .finish("initial_file_format", attribute_register.clone()),
            nexus_version: NexusAttribute::begin()
                .fixed_value("TODO".parse().expect(""))
                .finish("nexus_version", attribute_register.clone()),
            hdf_version: NexusAttribute::begin()
                .fixed_value("TODO".parse().expect(""))
                .finish("hdf_version", attribute_register.clone()),
            hdf5_version: NexusAttribute::begin()
                .fixed_value("TODO".parse().expect(""))
                .finish("hdf5_version", attribute_register.clone()),
            xml_version: NexusAttribute::begin()
                .fixed_value("TODO".parse().expect(""))
                .finish("xml_version", attribute_register.clone()),
            creator: NexusAttribute::begin()
                .fixed_value("TODO".parse().expect(""))
                .finish("creator", attribute_register.clone()),
        }
    }
}

pub(crate) struct NXRoot {
    raw_data_1: RcNexusGroup<raw_data::RawData>,
}

impl NxGroup for NXRoot {
    const CLASS_NAME: &'static str = nexus_class::ROOT;

    fn new(database_register: RcGroupContentRegister) -> Self {
        Self {
            raw_data_1: NexusGroup::new("raw_data_1", Some(database_register)),
        }
    }
}

impl<'a> NxPushMessage<FrameAssembledEventListMessage<'a>> for NXRoot {
    type MessageType = FrameAssembledEventListMessage<'a>;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.raw_data_1.push_message(message)
    }
}

impl<'a> NxPushMessage<RunStart<'a>> for NXRoot {
    type MessageType = RunStart<'a>;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.raw_data_1.push_message(message)
    }
}
impl<'a> NxPushMessage<RunStop<'a>> for NXRoot {
    type MessageType = RunStop<'a>;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.raw_data_1.push_message(message)
    }
}

impl<'a> NxPushMessageMut<Alarm<'a>> for NXRoot {
    type MessageType = Alarm<'a>;

    fn push_message_mut(&mut self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.raw_data_1.push_message_mut(message)
    }
}

impl<'a> NxPushMessageMut<se00_SampleEnvironmentData<'a>> for NXRoot {
    type MessageType = se00_SampleEnvironmentData<'a>;

    fn push_message_mut(&mut self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.raw_data_1.push_message_mut(message)
    }
}

impl<'a> NxPushMessageMut<f144_LogData<'a>> for NXRoot {
    type MessageType = f144_LogData<'a>;

    fn push_message_mut(&mut self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.raw_data_1.push_message_mut(message)
    }
}
