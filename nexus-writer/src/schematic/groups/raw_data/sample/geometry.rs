use hdf5::{types::VarLenAscii, Group};

use crate::schematic::elements::{
    dataset::{NexusDataset, RcNexusDatasetVar},
    group::{NexusGroup, NxGroup, RcDatasetRegister},
};

pub(super) struct Geometry {
    name: RcNexusDatasetVar<VarLenAscii>,
}

impl NxGroup for Geometry {
    const CLASS_NAME: &'static str = "NXperiod";

    fn new(dataset_register : RcDatasetRegister) -> Self {
        Self {
            name: NexusDataset::begin().finish("", dataset_register.clone()),
        }
    }
}
