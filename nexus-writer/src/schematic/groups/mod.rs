use hdf5::Group;
use raw_data::RawData;

use crate::{
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::elements::{attribute::NexusAttribute, group::NexusGroup},
};

use super::{
    elements::{
        attribute::{NexusAttributeFixed, NexusAttributeMut},
        traits::{
            NexusDataHolderFixed, NexusDataHolderScalarMutable, NexusDatasetDef, NexusGroupDef, NexusHandleMessage, NexusPushMessage
        },
    },
    nexus_class, H5String,
};

pub(super) mod log;
pub(crate) mod raw_data;

struct RawData1Attributes {
    file_name: NexusAttributeMut<H5String>,
    file_time: NexusAttributeMut<H5String>,
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
            file_name: NexusAttribute::new_with_auto_default("file_name"),
            file_time: NexusAttribute::new_with_auto_default("file_time"),
            initial_file_format: NexusAttribute::new_with_fixed_value("initial_file_format", "TODO".parse().expect("")),
            nexus_version: NexusAttribute::new_with_fixed_value("nexus_version", "TODO".parse().expect("")),
            hdf_version: NexusAttribute::new_with_fixed_value("hdf_version", "TODO".parse().expect("")),
            hdf5_version: NexusAttribute::new_with_fixed_value("hdf5_version", "TODO".parse().expect("")),
            xml_version: NexusAttribute::new_with_fixed_value("xml_version", "TODO".parse().expect("")),
            creator: NexusAttribute::new_with_fixed_value("creator", "TODO".parse().expect("")),
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
