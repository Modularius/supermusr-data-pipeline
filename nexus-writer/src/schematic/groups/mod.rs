use chrono::Utc;
use hdf5::Group;
use raw_data::RawData;

use crate::{
    elements::{
        attribute::{NexusAttribute, NexusAttributeFixed, NexusAttributeMut},
        group::NexusGroup,
        traits::{
            NexusDataHolderFixed, NexusDataHolderScalarMutable, NexusGroupDef, NexusHandleMessage,
            NexusPushMessage, StandardMessage,
        },
    },
    error::NexusPushError,
    nexus::{FrameParameters, NexusConfiguration, NexusSettings, PeriodParameters, RunStarted},
    schematic::{nexus_class, H5String},
};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm,
    ecs_f144_logdata_generated::f144_LogData, ecs_pl72_run_start_generated::RunStart,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};

pub(super) mod log;
pub(crate) mod raw_data;

pub(crate) struct NXRoot {
    /// file name of original data file to assist identification if the external name has been changed
    file_name: NexusAttributeMut<H5String, Group>,
    /// date and time of file creation
    file_time: NexusAttributeMut<H5String, Group>,
    /// Format used when creating initial NeXus file
    initial_file_format: NexusAttributeFixed<H5String, Group>,
    /// version of nexus API used in writing the file
    nexus_version: NexusAttributeFixed<H5String, Group>,
    /// version of HDF library used by nexus to create file
    hdf_version: NexusAttributeFixed<H5String, Group>,
    /// version of HDF5 library used by nexus to create file
    hdf5_version: NexusAttributeFixed<H5String, Group>,
    /// version of XML library used to create file
    xml_version: NexusAttributeFixed<H5String, Group>,
    /// facility or program where file originated
    creator: NexusAttributeFixed<H5String, Group>,
    /// Entries holding the raw data should follow the defined naming convention and be numbered;
    /// entries may also be written containing analysed data etc using a locally defined naming scheme.
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
                "HDF5".parse().expect(""),
            ),
            nexus_version: NexusAttribute::new_with_fixed_value(
                "nexus_version",
                "TODO".parse().expect(""), // Where does this come from?
            ),
            hdf_version: NexusAttribute::new_with_fixed_value(
                "hdf_version",
                "1.14.3".parse().expect(""), // Is this the same as for hdf5 version?
            ),
            hdf5_version: NexusAttribute::new_with_fixed_value(
                "hdf5_version",
                "1.14.3".parse().expect(""), // Can this be taken directly from the nix package;
            ),
            xml_version: NexusAttribute::new_with_fixed_value(
                "xml_version",
                "N/A".parse().expect(""),
            ),
            creator: NexusAttribute::new_with_fixed_value(
                "creator",
                "SuperMuSR Data Pipeline Nexus Writer".parse().expect(""),
            ),
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>, Group, RunStarted> for NXRoot {
    fn handle_message(
        &mut self,
        message: &RunStart<'a>,
        parent: &Group,
    ) -> Result<RunStarted, NexusPushError> {
        if let Some(filename) = message.filename() {
            self.file_name
                .write_scalar(parent, filename.parse().expect(""))?;
        }
        self.file_time
            .write_scalar(parent, Utc::now().to_rfc3339().parse().expect(""))?;
        self.initial_file_format.write(parent)?;
        self.nexus_version.write(parent)?;
        self.hdf_version.write(parent)?;
        self.hdf5_version.write(parent)?;
        self.xml_version.write(parent)?;
        self.creator.write(parent)?;
        self.raw_data_1.push_message(message, parent)
    }
}

impl StandardMessage<NexusConfiguration> for NXRoot {}
impl<'a> StandardMessage<RunStop<'a>> for NXRoot {}
impl<'a> StandardMessage<FrameAssembledEventListMessage<'a>> for NXRoot {}
impl<'a> StandardMessage<Alarm<'a>> for NXRoot {}
impl<'a> StandardMessage<se00_SampleEnvironmentData<'a>> for NXRoot {}
impl<'a> StandardMessage<f144_LogData<'a>> for NXRoot {}
impl StandardMessage<Vec<FrameParameters>> for NXRoot {}
impl StandardMessage<Vec<PeriodParameters>> for NXRoot {}

impl<M, R> NexusHandleMessage<M, Group, R> for NXRoot
where
    Self: StandardMessage<M>,
    RawData: NexusHandleMessage<M, Group, R>,
{
    fn handle_message(&mut self, message: &M, parent: &Group) -> Result<R, NexusPushError> {
        self.raw_data_1.push_message(message, parent)
    }
}
