use crate::{
    elements::{
        dataset::{NexusDataset, NexusDatasetMut},
        traits::{NexusDataHolderScalarMutable, NexusGroupDef},
    },
    nexus::NexusSettings,
    schematic::{nexus_class, H5String},
};

pub(super) struct Geometry {
    name: NexusDatasetMut<H5String>,
}

impl NexusGroupDef for Geometry {
    const CLASS_NAME: &'static str = nexus_class::GEOMETRY;
    type Settings = NexusSettings;

    fn new(_settings: &NexusSettings) -> Self {
        Self {
            name: NexusDataset::new_with_default("name"),
        }
    }
}
