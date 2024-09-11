use hdf5::{File, Group, Location};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm,
    ecs_f144_logdata_generated::f144_LogData, ecs_pl72_run_start_generated::RunStart,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};

use crate::schematic::elements::{attribute::NexusAttribute, group::NexusGroup};

use super::{
    elements::{
        attribute::NexusAttributeFixed, NexusBuildable, NexusBuilderFinished, NexusDatasetDef,
        NexusError, NexusGroupDef, NexusPushMessage, NexusPushMessageMut,
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

impl NexusDatasetDef for RawData1Attributes {
    fn new() -> Self {
        Self {
            file_name: NexusAttribute::begin("file_name")
                .default_value(Default::default())
                .finish(),
            file_time: NexusAttribute::begin("file_time")
                .default_value(Default::default())
                .finish(),
            initial_file_format: NexusAttribute::begin("initial_file_format")
                .fixed_value("TODO".parse().expect(""))
                .finish(),
            nexus_version: NexusAttribute::begin("nexus_version")
                .fixed_value("TODO".parse().expect(""))
                .finish(),
            hdf_version: NexusAttribute::begin("hdf_version")
                .fixed_value("TODO".parse().expect(""))
                .finish(),
            hdf5_version: NexusAttribute::begin("hdf5_version")
                .fixed_value("TODO".parse().expect(""))
                .finish(),
            xml_version: NexusAttribute::begin("xml_version")
                .fixed_value("TODO".parse().expect(""))
                .finish(),
            creator: NexusAttribute::begin("creator")
                .fixed_value("TODO".parse().expect(""))
                .finish(),
        }
    }
}

pub(crate) struct NXRoot {
    raw_data_1: NexusGroup<raw_data::RawData>,
}

impl NexusGroupDef for NXRoot {
    const CLASS_NAME: &'static str = nexus_class::ROOT;

    fn new() -> Self {
        Self {
            raw_data_1: NexusGroup::new("raw_data_1"),
        }
    }
}

impl<'a> NexusPushMessage<File, FrameAssembledEventListMessage<'a>> for NXRoot {
    fn push_message(&self, message: &FrameAssembledEventListMessage<'a>, parent: &File) -> Result<(), NexusError> {
        let group = self.raw_data_1.create_hdf5(&parent)?;
        self.raw_data_1.push_message(message, &group)
    }
}

impl<'a> NexusPushMessage<File, RunStart<'a>> for NXRoot {
    fn push_message(&self, message: &RunStart<'a>, parent: &File) -> Result<(), NexusError> {
        let group = self.raw_data_1.create_hdf5(&parent)?;
        self.raw_data_1.push_message(message, &group)
    }
}

impl<'a> NexusPushMessage<File, RunStop<'a>> for NXRoot {
    fn push_message(&self, message: &RunStop<'a>, parent: &File) -> Result<(), NexusError> {
        let group = self.raw_data_1.create_hdf5(&parent)?;
        self.raw_data_1.push_message(message, &group)
    }
}

impl<'a> NexusPushMessageMut<File, Alarm<'a>> for NXRoot {
    fn push_message_mut(&mut self, message: &Alarm<'a>, parent: &File) -> Result<(), NexusError> {
        let group = self.raw_data_1.create_hdf5(&parent)?;
        self.raw_data_1.push_message_mut(message, &group)
    }
}

impl<'a> NexusPushMessageMut<File, se00_SampleEnvironmentData<'a>> for NXRoot {
    fn push_message_mut(&mut self, message: &se00_SampleEnvironmentData<'a>, parent: &File) -> Result<(), NexusError> {
        let group = self.raw_data_1.create_hdf5(&parent)?;
        self.raw_data_1.push_message_mut(message, &group)
    }
}

impl<'a> NexusPushMessageMut<File, f144_LogData<'a>> for NXRoot {
    fn push_message_mut(&mut self, message: &f144_LogData<'a>, parent: &File) -> Result<(), NexusError> {
        let group = self.raw_data_1.create_hdf5(&parent)?;
        self.raw_data_1.push_message_mut(message, &group)
    }
}
