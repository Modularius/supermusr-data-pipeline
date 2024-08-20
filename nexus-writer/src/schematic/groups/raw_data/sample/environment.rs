use hdf5::types::VarLenAscii;

use crate::schematic::elements::{dataset::NexusDataset, NexusGroup, NxGroup};


pub(super) struct Environment {
    name: NexusDataset<u32>,
}

impl NxGroup for Environment {
    const CLASS_NAME : &'static str = "NXperiod";

    fn new() -> Self {
        Self {
            name: NexusDataset::new(""),
        }
    }
}