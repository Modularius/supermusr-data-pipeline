use crate::schematic::{
    elements::{
        dataset::{Buildable, NexusDataset},
        group::{NexusGroup, NxGroup, RcGroupContentRegister, RcNexusGroup},
    },
    groups::log::Log,
    nexus_class, H5String,
};

pub(super) struct Source {
    name: NexusDataset<H5String>,
    source_type: NexusDataset<H5String>,
    probe: NexusDataset<H5String>,
    source_frequency: NexusDataset<H5String>,
    source_frame_pattern: NexusDataset<H5String>,
    source_energy: NexusDataset<H5String>,
    source_current: NexusDataset<H5String>,
    source_current_log: RcNexusGroup<Log>,
    source_pulse_width: NexusDataset<H5String>,
    target_material: NexusDataset<H5String>,
    target_thickness: NexusDataset<H5String>,
}

impl NxGroup for Source {
    const CLASS_NAME: &'static str = nexus_class::SOURCE;

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            name: NexusDataset::begin().finish("name", dataset_register.clone()),
            source_type: NexusDataset::begin().finish("source_type", dataset_register.clone()),
            probe: NexusDataset::begin().finish("probe", dataset_register.clone()),
            source_frequency: NexusDataset::begin()
                .finish("source_frequency", dataset_register.clone()),
            source_frame_pattern: NexusDataset::begin()
                .finish("source_frame_pattern", dataset_register.clone()),
            source_energy: NexusDataset::begin().finish("source_energy", dataset_register.clone()),
            source_current: NexusDataset::begin()
                .finish("source_current", dataset_register.clone()),
            source_current_log: NexusGroup::new("source_current_log", None),
            source_pulse_width: NexusDataset::begin()
                .finish("source_pulse_width", dataset_register.clone()),
            target_material: NexusDataset::begin()
                .finish("target_material", dataset_register.clone()),
            target_thickness: NexusDataset::begin()
                .finish("target_thickness", dataset_register.clone()),
        }
    }
}
