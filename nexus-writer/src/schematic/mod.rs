mod dataset;
mod group;

use std::{fs::create_dir_all, marker::PhantomData, ops::Deref};
use chrono::{DateTime,Utc};
use dataset::{NexusDatasetData, Units};
use group::{nx_group, NexusGroup};
use hdf5::{plist::DatasetCreateBuilder, Attribute, AttributeBuilder, Dataset, File, Group};

pub(crate) struct NexusFile {
    file: File,
    raw_data_1: NexusGroup<nx_group::RawData1>,
}

impl NexusFile {
    fn new(filename: &str, run_name: &str) {
        create_dir_all(filename)?;
        let filename = {
            let mut filename = filename.to_owned();
            filename.push(run_name);
            filename.set_extension("nxs");
            filename
        };

        let file = File::create(filename)?;
        let raw_data_1 = NexusGroup::new(file);
        Self {
            file,
            raw_data_1
        }
    }
}


