use hdf5::types::VarLenAscii;

use crate::schematic::elements::{dataset::NexusDataset, NexusGroup, NxGroup};


pub(super) struct Instrument {
    name: NexusDataset<VarLenAscii>,
}

impl NxGroup for Instrument {
    const CLASS_NAME : &'static str = "NXperiod";

    fn new() -> Self {
        Self {
            name: NexusDataset::new(""),
        }
    }
}