use hdf5::Group;
use raw_data::RawData;

use crate::{
    nexus::NexusSettings,
    schematic::elements::{attribute::NexusAttribute, group::NexusGroup},
};

use super::{
    elements::{
        attribute::NexusAttributeFixed, NexusBuildable, NexusBuilderFinished, NexusDatasetDef,
        NexusError, NexusGroupDef, NexusHandleMessage, NexusHandleMessageWithContext,
        NexusPushMessage, NexusPushMessageWithContext,
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
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            raw_data_1: NexusGroup::new("raw_data_1", settings),
        }
    }
}

impl<M> NexusHandleMessage<M, Group> for NXRoot
where
    RawData: NexusHandleMessage<M>,
{
    fn handle_message(&mut self, message: &M, parent: &Group) -> Result<(), NexusError> {
        self.raw_data_1.push_message(message, parent)
    }
}

impl<M, Ctxt> NexusHandleMessageWithContext<M, Group> for NXRoot
where
    RawData: NexusHandleMessageWithContext<M, Context = Ctxt>,
{
    type Context = Ctxt;

    fn handle_message_with_context(
        &mut self,
        message: &M,
        parent: &Group,
        context: &mut Self::Context,
    ) -> Result<(), NexusError> {
        self.raw_data_1
            .push_message_with_context(message, parent, context)
    }
}
