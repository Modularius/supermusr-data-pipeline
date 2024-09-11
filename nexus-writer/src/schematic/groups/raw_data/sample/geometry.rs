use crate::schematic::{
    elements::{dataset::NexusDataset, NexusBuildable, NexusBuilderFinished, NexusGroupDef},
    nexus_class, H5String,
};

pub(super) struct Geometry {
    name: NexusDataset<H5String>,
}

impl NexusGroupDef for Geometry {
    const CLASS_NAME: &'static str = nexus_class::GEOMETRY;

    fn new() -> Self {
        Self {
            name: NexusDataset::begin("name")
                .default_value(Default::default())
                .finish(),
        }
    }
}
