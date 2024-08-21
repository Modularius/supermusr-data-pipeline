use chrono::{DateTime, Utc};
use hdf5::{
    types::{FixedAscii, TypeDescriptor, VarLenAscii},
    Group,
};
use supermusr_streaming_types::{aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage, ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm, ecs_f144_logdata_generated::f144_LogData, ecs_pl72_run_start_generated::RunStart, ecs_se00_data_generated::se00_SampleEnvironmentData};

use crate::schematic::elements::{
    attribute::NexusAttribute,
    group::{NexusGroup, NxGroup},
};

use super::elements::group::NxPushMessage;

pub(super) mod log;
pub(crate) mod raw_data;

pub(crate) struct NXRoot {
    file_name: NexusAttribute,
    file_time: NexusAttribute,
    initial_file_format: NexusAttribute,
    nexus_version: NexusAttribute,
    hdf_version: NexusAttribute,
    hdf5_version: NexusAttribute,
    xml_version: NexusAttribute,
    creator: NexusAttribute,
    raw_data_1: NexusGroup<raw_data::RawData>,
}

impl NxGroup for NXRoot {
    const CLASS_NAME: &'static str = "NXroot";

    fn new() -> Self {
        Self {
            file_name: NexusAttribute::new("file_name", TypeDescriptor::VarLenAscii),
            file_time: NexusAttribute::new("file_time", TypeDescriptor::VarLenAscii),
            initial_file_format: NexusAttribute::new(
                "initial_file_format",
                TypeDescriptor::VarLenAscii,
            ),
            nexus_version: NexusAttribute::new("nexus_version", TypeDescriptor::VarLenAscii),
            hdf_version: NexusAttribute::new("hdf_version", TypeDescriptor::VarLenAscii),
            hdf5_version: NexusAttribute::new("hdf5_version", TypeDescriptor::VarLenAscii),
            xml_version: NexusAttribute::new("xml_version", TypeDescriptor::VarLenAscii),
            creator: NexusAttribute::new("creator", TypeDescriptor::VarLenAscii),
            raw_data_1: NexusGroup::new("raw_data_1"),
        }
    }

    fn create(&mut self, this: &Group) {
        self.raw_data_1.create(this);
        let this = self.raw_data_1.get_group().expect("Group Exists");
        self.file_name.create(this);
        self.file_time.create(this);
        self.initial_file_format.create(this);
        self.nexus_version.create(this);
        self.hdf_version.create(this);
        self.hdf5_version.create(this);
        self.xml_version.create(this);
    }

    fn open(&mut self, this: &Group) {
        self.raw_data_1.open(this);
        let this = self.raw_data_1.get_group().expect("Group Exists");
        self.file_name.open(this);
        self.file_time.open(this);
        self.initial_file_format.open(this);
        self.nexus_version.open(this);
        self.hdf_version.open(this);
        self.hdf5_version.open(this);
        self.xml_version.open(this);
    }

    fn close(&mut self) {
        self.raw_data_1.close();
        self.file_name.close();
        self.file_time.close();
        self.initial_file_format.close();
        self.nexus_version.close();
        self.hdf_version.close();
        self.hdf5_version.close();
        self.xml_version.close();
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