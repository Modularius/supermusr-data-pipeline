
use std::{fs::create_dir_all, marker::PhantomData, ops::Deref};
use chrono::{DateTime,Utc};
//use dataset::{NexusDatasetData, Units};
//use group::{nx_group, NexusGroup};
use hdf5::{plist::DatasetCreateBuilder, types::{FixedAscii, VarLenAscii}, Attribute, AttributeBuilder, Dataset, File, Group};

use super::elements::{attribute::NexusAttribute, dataset::{NexusDataset, NexusUnits}, NexusGroup, NxGroup};

pub(crate) struct NXRoot {
    file_name: NexusAttribute,
    file_time: NexusAttribute,
    initial_file_format: NexusAttribute,
    nexus_version: NexusAttribute,
    hdf_version: NexusAttribute,
    hdf5_version: NexusAttribute,
    xml_version: NexusAttribute,
    creator: NexusAttribute,
    raw_data_1: NexusGroup<RawData>,
}

impl NxGroup for NXRoot {
    const CLASS_NAME : &'static str = "NXroot";

    fn new() -> Self {
        Self {
            file_name: NexusAttribute::new(),
            file_time: NexusAttribute::new(),
            initial_file_format: NexusAttribute::new(),
            nexus_version: NexusAttribute::new(),
            hdf_version: NexusAttribute::new(),
            hdf5_version: NexusAttribute::new(),
            xml_version: NexusAttribute::new(),
            creator: NexusAttribute::new(),
            raw_data_1: NexusGroup::new(),
        }
    }
}




struct RunLog {
}

impl NxGroup for RunLog {
    const CLASS_NAME : &'static str = "NXrunlog";

    fn new() -> Self {
        Self {
        }
    }
}


struct Selog {
}

impl NxGroup for Selog {
    const CLASS_NAME : &'static str = "NXselog";

    fn new() -> Self {
        Self {
        }
    }
}

