use crate::schematic::{
    elements::{
        dataset::NexusDataset,traits::Buildable,
        group::{NxGroup, RcGroupContentRegister},
    },
    nexus_class, H5String,
};

pub(super) struct Geometry {
    name: NexusDataset<H5String>,
}

impl NxGroup for Geometry {
    const CLASS_NAME: &'static str = nexus_class::GEOMETRY;

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            name: NexusDataset::begin("name").finish(&dataset_register),
        }
    }
}
