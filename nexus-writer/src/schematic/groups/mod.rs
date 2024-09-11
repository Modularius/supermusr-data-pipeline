use chrono::Local;
use hdf5::{File, Group, Location};
use raw_data::RawData;
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm,
    ecs_f144_logdata_generated::f144_LogData, ecs_pl72_run_start_generated::RunStart,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};

use crate::schematic::elements::{attribute::NexusAttribute, group::NexusGroup};

use super::{
    elements::{
        attribute::NexusAttributeFixed, NexusBuildable, NexusBuilderFinished, NexusDatasetDef, NexusError, NexusGroupDef, NexusHandleMessage, NexusPushMessage
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

impl<M> NexusHandleMessage<M,Group> for NXRoot where RawData : NexusHandleMessage<M> {
    fn handle_message(&mut self, message: &M, parent: &Group) -> Result<(), NexusError> {
        self.raw_data_1.push_message(message, parent)
    }
}
