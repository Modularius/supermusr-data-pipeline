use chrono::{DateTime,Utc};
use hdf5::{types::{FixedAscii, TypeDescriptor, VarLenAscii}, Attribute, AttributeBuilder, Dataset, File, Group};

use crate::schematic::elements::{attribute::NexusAttribute, dataset::{NexusDataset, NexusUnits}, NexusGroup, NxGroup};

pub(crate) mod raw_data;
pub(super) mod log;


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
    const CLASS_NAME : &'static str = "NXroot";

    fn new() -> Self {
        Self {
            file_name: NexusAttribute::new("file_name", TypeDescriptor::VarLenAscii),
            file_time: NexusAttribute::new("file_time", TypeDescriptor::VarLenAscii),
            initial_file_format: NexusAttribute::new("initial_file_format", TypeDescriptor::VarLenAscii),
            nexus_version: NexusAttribute::new("nexus_version", TypeDescriptor::VarLenAscii),
            hdf_version: NexusAttribute::new("hdf_version", TypeDescriptor::VarLenAscii),
            hdf5_version: NexusAttribute::new("hdf5_version", TypeDescriptor::VarLenAscii),
            xml_version: NexusAttribute::new("xml_version", TypeDescriptor::VarLenAscii),
            creator: NexusAttribute::new("creator", TypeDescriptor::VarLenAscii),
            raw_data_1: NexusGroup::new("raw_data_1"),
        }
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