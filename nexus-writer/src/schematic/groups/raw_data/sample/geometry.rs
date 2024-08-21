use hdf5::{types::VarLenAscii, Group};

use crate::schematic::elements::{
    dataset::NexusDataset,
    group::{NexusGroup, NxGroup},
};

pub(super) struct Geometry {
    name: NexusDataset<VarLenAscii>,
}

impl NxGroup for Geometry {
    const CLASS_NAME: &'static str = "NXperiod";

    fn new() -> Self {
        Self {
            name: NexusDataset::begin().finish(""),
        }
    }

    fn create(&mut self, this: &Group) {
        self.name.create(this);
    }

    fn open(&mut self, this: &Group) {
        self.name.open(this);
    }

    fn close(&mut self) {
        self.name.close();
    }
}
