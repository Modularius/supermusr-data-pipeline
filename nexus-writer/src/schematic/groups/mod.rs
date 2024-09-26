use hdf5::Group;
use raw_data::RawData;

use crate::{
    elements::{
        attribute::{NexusAttribute, NexusAttributeFixed, NexusAttributeMut},
        group::NexusGroup,
        traits::{
            NexusDataHolderFixed, NexusDataHolderScalarMutable, NexusDatasetDef, NexusGroupDef,
            NexusHandleMessage, NexusPushMessage,
        },
    },
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{nexus_class, H5String},
};

pub(super) mod log;
pub(crate) mod raw_data;

struct RawData1Attributes {
    _file_name: NexusAttributeMut<H5String>,
    _file_time: NexusAttributeMut<H5String>,
    _initial_file_format: NexusAttributeFixed<H5String>,
    _nexus_version: NexusAttributeFixed<H5String>,
    _hdf_version: NexusAttributeFixed<H5String>,
    _hdf5_version: NexusAttributeFixed<H5String>,
    _xml_version: NexusAttributeFixed<H5String>,
    _creator: NexusAttributeFixed<H5String>,
}

impl NexusDatasetDef for RawData1Attributes {
    fn new() -> Self {
        Self {
            _file_name: NexusAttribute::new_with_default("file_name"),
            _file_time: NexusAttribute::new_with_default("file_time"),
            _initial_file_format: NexusAttribute::new_with_fixed_value(
                "initial_file_format",
                "TODO".parse().expect(""),
            ),
            _nexus_version: NexusAttribute::new_with_fixed_value(
                "nexus_version",
                "TODO".parse().expect(""),
            ),
            _hdf_version: NexusAttribute::new_with_fixed_value(
                "hdf_version",
                "TODO".parse().expect(""),
            ),
            _hdf5_version: NexusAttribute::new_with_fixed_value(
                "hdf5_version",
                "TODO".parse().expect(""),
            ),
            _xml_version: NexusAttribute::new_with_fixed_value(
                "xml_version",
                "TODO".parse().expect(""),
            ),
            _creator: NexusAttribute::new_with_fixed_value("creator", "TODO".parse().expect("")),
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
