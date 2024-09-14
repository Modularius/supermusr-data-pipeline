use crate::{
    nexus::NexusSettings,
    schematic::{
        elements::{dataset::NexusDataset, NexusBuildable, NexusGroupDef},
        nexus_class, H5String,
    },
};

pub(super) struct Geometry {
    name: NexusDataset<H5String>,
}

impl NexusGroupDef for Geometry {
    const CLASS_NAME: &'static str = nexus_class::GEOMETRY;
    type Settings = NexusSettings;

    fn new(_settings: &NexusSettings) -> Self {
        Self {
            name: NexusDataset::begin("name").finish_with_auto_default(),
        }
    }
}
