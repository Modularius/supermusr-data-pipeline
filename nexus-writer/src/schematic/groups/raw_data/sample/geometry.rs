use hdf5::types::VarLenAscii;

use crate::schematic::elements::{dataset::NexusDataset, NexusGroup, NxGroup};


pub(super) struct Geometry {
    name: NexusDataset<VarLenAscii>,
}

impl NxGroup for Geometry {
    const CLASS_NAME : &'static str = "NXperiod";

    fn new() -> Self {
        Self {
            name: NexusDataset::new(""),
        }
    }
}