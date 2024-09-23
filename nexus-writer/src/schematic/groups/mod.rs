use hdf5::Group;
use raw_data::RawData;

use crate::{
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::elements::{attribute::NexusAttribute, group::NexusGroup},
};

use super::{
    elements::{
        attribute::NexusAttributeFixed, traits::{NexusBuildable, NexusDatasetDef, NexusGroupDef,
        NexusHandleMessage, NexusPushMessage}
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
            file_name: NexusAttribute::begin("file_name").finish_with_auto_default(),
            file_time: NexusAttribute::begin("file_time").finish_with_auto_default(),
            initial_file_format: NexusAttribute::begin("initial_file_format")
                .finish_with_fixed_value("TODO".parse().expect("")),
            nexus_version: NexusAttribute::begin("nexus_version")
                .finish_with_fixed_value("TODO".parse().expect("")),
            hdf_version: NexusAttribute::begin("hdf_version")
                .finish_with_fixed_value("TODO".parse().expect("")),
            hdf5_version: NexusAttribute::begin("hdf5_version")
                .finish_with_fixed_value("TODO".parse().expect("")),
            xml_version: NexusAttribute::begin("xml_version")
                .finish_with_fixed_value("TODO".parse().expect("")),
            creator: NexusAttribute::begin("creator")
                .finish_with_fixed_value("TODO".parse().expect("")),
        }
    }
}

pub(crate) struct NXRoot {
    raw_data_1: NexusGroup<raw_data::RawData>,
}

impl NexusGroupDef for NXRoot {
    const CLASS_NAME: &'static str = nexus_class::ROOT;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            raw_data_1: NexusGroup::new("raw_data_1", settings),
        }
    }
}

impl<M, R> NexusHandleMessage<M, Group, R> for NXRoot
where
    RawData: NexusHandleMessage<M, Group, R>,
{
    fn handle_message(&mut self, message: &M, parent: &Group) -> Result<R, NexusPushError> {
        self.raw_data_1.push_message(message, parent)
    }
}

impl<M, Ctxt, R> NexusHandleMessageWithContext<M, Group, R> for NXRoot
where
    RawData: NexusHandleMessageWithContext<M, Group, R, Context = Ctxt>,
{
    type Context = Ctxt;

    fn handle_message_with_context(
        &mut self,
        message: &M,
        parent: &Group,
        context: &mut Self::Context,
    ) -> Result<R, NexusPushError> {
        self.raw_data_1
            .push_message_with_context(message, parent, context)
    }
}
