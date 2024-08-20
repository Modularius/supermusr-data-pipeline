use hdf5::types::VarLenAscii;

use crate::schematic::elements::{dataset::NexusDataset, NexusGroup, NxGroup};


pub(super) struct RunLog {
    name: NexusDataset<VarLenAscii>,
}

impl NxGroup for RunLog {
    const CLASS_NAME : &'static str = "NXperiod";

    fn new() -> Self {
        Self {
            name: NexusDataset::new(""),
        }
    }
}