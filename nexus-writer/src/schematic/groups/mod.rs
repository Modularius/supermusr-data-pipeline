use chrono::Utc;
use hdf5::Group;
use raw_data::RawData;

use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm,
    ecs_f144_logdata_generated::f144_LogData, ecs_pl72_run_start_generated::RunStart,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};
use crate::{
    elements::{
        attribute::{NexusAttribute, NexusAttributeFixed, NexusAttributeMut}, group::NexusGroup, traits::{
            NexusDataHolderFixed, NexusDataHolderScalarMutable, NexusDatasetDef, NexusGroupDef,
            NexusHandleMessage, NexusPushMessage,
        }
    },
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{nexus_class, H5String},
};

pub(super) mod log;
pub(crate) mod raw_data;

pub(crate) struct NXRoot {
    file_name: NexusAttributeMut<H5String, Group>,
    file_time: NexusAttributeMut<H5String, Group>,
    initial_file_format: NexusAttributeFixed<H5String, Group>,
    nexus_version: NexusAttributeFixed<H5String, Group>,
    hdf_version: NexusAttributeFixed<H5String, Group>,
    hdf5_version: NexusAttributeFixed<H5String, Group>,
    xml_version: NexusAttributeFixed<H5String, Group>,
    creator: NexusAttributeFixed<H5String, Group>,
    raw_data_1: NexusGroup<raw_data::RawData>,
}

impl NexusGroupDef for NXRoot {
    const CLASS_NAME: &'static str = nexus_class::ROOT;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            raw_data_1: NexusGroup::new("raw_data_1", settings),
            file_name: NexusAttribute::new_with_default("file_name"),
            file_time: NexusAttribute::new_with_default("file_time"),
            initial_file_format: NexusAttribute::new_with_fixed_value(
                "initial_file_format",
                "TODO".parse().expect(""),
            ),
            nexus_version: NexusAttribute::new_with_fixed_value(
                "nexus_version",
                "TODO".parse().expect(""),// Where does this come from?
            ),
            hdf_version: NexusAttribute::new_with_fixed_value(
                "hdf_version",
                "1.14.3".parse().expect(""),// Is this the same as for hdf5 version?
            ),
            hdf5_version: NexusAttribute::new_with_fixed_value(
                "hdf5_version",
                "1.14.3".parse().expect(""),// Can this be taken directly from the nix package;
            ),
            xml_version: NexusAttribute::new_with_fixed_value(
                "xml_version",
                "TODO".parse().expect(""),
            ),
            creator: NexusAttribute::new_with_fixed_value("creator", "TODO".parse().expect("")),
        }
    }
}

impl<'a,R> NexusHandleMessage<RunStart<'a>, Group, R> for NXRoot
where
    RawData: NexusHandleMessage<RunStart<'a>, Group, R>,
{
    fn handle_message(&mut self, message: &RunStart<'a>, parent: &Group) -> Result<R, NexusPushError> {
        if let Some(filename) = message.filename() {
            self.file_name.write_scalar(parent, filename.parse().expect(""))?;
        }
        self.file_time.write_scalar(parent, Utc::now().to_rfc3339().parse().expect(""))?;
        self.initial_file_format.write(parent)?;
        self.nexus_version.write(parent)?;
        self.hdf_version.write(parent)?;
        self.hdf5_version.write(parent)?;
        self.xml_version.write(parent)?;
        self.creator.write(parent)?;
        self.raw_data_1.push_message(message, parent)
    }
}


impl<'a, R> NexusHandleMessage<RunStop<'a>, Group, R> for NXRoot
where
    RawData: NexusHandleMessage<RunStop<'a>, Group, R>,
{
    fn handle_message(&mut self, message: &RunStop<'a>, parent: &Group) -> Result<R, NexusPushError> {
        self.raw_data_1.push_message(message, parent)
    }
}

impl<'a, R> NexusHandleMessage<FrameAssembledEventListMessage<'a>, Group, R> for NXRoot
where
    RawData: NexusHandleMessage<FrameAssembledEventListMessage<'a>, Group, R>,
{
    fn handle_message(&mut self, message: &FrameAssembledEventListMessage<'a>, parent: &Group) -> Result<R, NexusPushError> {
        self.raw_data_1.push_message(message, parent)
    }
}

impl<'a, R> NexusHandleMessage<Alarm<'a>, Group, R> for NXRoot
where
    RawData: NexusHandleMessage<Alarm<'a>, Group, R>,
{
    fn handle_message(&mut self, message: &Alarm<'a>, parent: &Group) -> Result<R, NexusPushError> {
        self.raw_data_1.push_message(message, parent)
    }
}

impl<'a, R> NexusHandleMessage<f144_LogData<'a>, Group, R> for NXRoot
where
    RawData: NexusHandleMessage<f144_LogData<'a>, Group, R>,
{
    fn handle_message(&mut self, message: &f144_LogData<'a>, parent: &Group) -> Result<R, NexusPushError> {
        self.raw_data_1.push_message(message, parent)
    }
}

impl<'a, R> NexusHandleMessage<se00_SampleEnvironmentData<'a>, Group, R> for NXRoot
where
    RawData: NexusHandleMessage<se00_SampleEnvironmentData<'a>, Group, R>,
{
    fn handle_message(&mut self, message: &se00_SampleEnvironmentData<'a>, parent: &Group) -> Result<R, NexusPushError> {
        self.raw_data_1.push_message(message, parent)
    }
}
/*
impl<M, R> NexusHandleMessage<M, Group, R> for NXRoot
where
    RawData: NexusHandleMessage<M, Group, R>,
{
    fn handle_message(&mut self, message: &M, parent: &Group) -> Result<R, NexusPushError> {
        self.raw_data_1.push_message(message, parent)
    }
}
 */