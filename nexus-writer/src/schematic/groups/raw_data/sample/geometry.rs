use crate::schematic::{
    elements::{
        dataset::NexusDataset,
        group::{GroupContentRegister, NxGroup},
        traits::Buildable,
    },
    nexus_class, H5String,
};

pub(super) struct Geometry {
    name: NexusDataset<H5String>,
}

impl NxGroup for Geometry {
    const CLASS_NAME: &'static str = nexus_class::GEOMETRY;

    fn new(dataset_register: GroupContentRegister) -> Self {
        Self {
            name: NexusDataset::begin("name").finish(&dataset_register),
        }
    }
}
