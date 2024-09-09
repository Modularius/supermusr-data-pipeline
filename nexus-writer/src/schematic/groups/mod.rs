use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm,
    ecs_f144_logdata_generated::f144_LogData, ecs_pl72_run_start_generated::RunStart,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};

use crate::{
    nexus::Run,
    schematic::elements::{
        attribute::NexusAttribute,
        group::{NexusGroup, NxGroup},
    },
};

use super::{
    elements::{
        attribute::NexusAttributeFixed,
        dataset::{AttributeRegister, NxDataset},
        group::{GroupContentRegister, NxPushMessage, NxPushMessageMut},
        traits::{Buildable, SubgroupBuildable},
    },
    nexus_class, H5String,
};

pub(super) mod log;
pub(crate) mod raw_data;

struct RawData1Attributes {
    file_name: NexusAttribute<H5String>,
    file_time: NexusAttribute<H5String>,
    initial_file_format: NexusAttributeFixed<H5String>,
    nexus_version: NexusAttributeFixed<H5String>,
    hdf_version: NexusAttributeFixed<H5String>,
    hdf5_version: NexusAttributeFixed<H5String>,
    xml_version: NexusAttributeFixed<H5String>,
    creator: NexusAttributeFixed<H5String>,
}

impl NxDataset for RawData1Attributes {
    fn new(attribute_register: AttributeRegister) -> Self {
        Self {
            file_name: NexusAttribute::begin("file_name")
                .default_value(Default::default())
                .finish(&attribute_register),
            file_time: NexusAttribute::begin("file_time")
                .default_value(Default::default())
                .finish(&attribute_register),
            initial_file_format: NexusAttribute::begin("initial_file_format")
                .fixed_value("TODO".parse().expect(""))
                .finish(&attribute_register),
            nexus_version: NexusAttribute::begin("nexus_version")
                .fixed_value("TODO".parse().expect(""))
                .finish(&attribute_register),
            hdf_version: NexusAttribute::begin("hdf_version")
                .fixed_value("TODO".parse().expect(""))
                .finish(&attribute_register),
            hdf5_version: NexusAttribute::begin("hdf5_version")
                .fixed_value("TODO".parse().expect(""))
                .finish(&attribute_register),
            xml_version: NexusAttribute::begin("xml_version")
                .fixed_value("TODO".parse().expect(""))
                .finish(&attribute_register),
            creator: NexusAttribute::begin("creator")
                .fixed_value("TODO".parse().expect(""))
                .finish(&attribute_register),
        }
    }
}

pub(crate) struct NXRoot {
    raw_data_1: NexusGroup<raw_data::RawData>,
}

impl NxGroup for NXRoot {
    const CLASS_NAME: &'static str = nexus_class::ROOT;

    fn new(database_register: GroupContentRegister) -> Self {
        Self {
            raw_data_1: NexusGroup::new_subgroup("raw_data_1", &database_register),
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
