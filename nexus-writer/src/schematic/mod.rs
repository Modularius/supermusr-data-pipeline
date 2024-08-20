mod groups;
mod elements;


use groups::NXRoot;
//use dataset::{NexusDatasetData, Units};
//use group::{nx_group, NexusGroup};
use hdf5::File;


struct Nexus {
    file: File,
    nx_root: NXRoot
}