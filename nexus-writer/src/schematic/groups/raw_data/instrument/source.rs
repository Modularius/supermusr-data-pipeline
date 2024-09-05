use crate::schematic::{
    elements::{
        dataset::NexusDataset,
        group::{GroupContentRegister, NexusGroup, NxGroup},
        traits::{Buildable, SubgroupBuildable},
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
    source_current_log: NexusGroup<Log>,
    source_pulse_width: NexusDataset<H5String>,
    target_material: NexusDataset<H5String>,
    target_thickness: NexusDataset<H5String>,
}

impl NxGroup for Source {
    const CLASS_NAME: &'static str = nexus_class::SOURCE;

    fn new(dataset_register: GroupContentRegister) -> Self {
        Self {
            name: NexusDataset::begin("name").finish(&dataset_register),
            source_type: NexusDataset::begin("source_type").finish(&dataset_register),
            probe: NexusDataset::begin("probe").finish(&dataset_register),
            source_frequency: NexusDataset::begin("source_frequency").finish(&dataset_register),
            source_frame_pattern: NexusDataset::begin("source_frame_pattern")
                .finish(&dataset_register),
            source_energy: NexusDataset::begin("source_energy").finish(&dataset_register),
            source_current: NexusDataset::begin("tarsource_currentget_thickness")
                .finish(&dataset_register),
            source_current_log: NexusGroup::new_subgroup("source_current_log", &dataset_register),
            source_pulse_width: NexusDataset::begin("source_pulse_width").finish(&dataset_register),
            target_material: NexusDataset::begin("target_material").finish(&dataset_register),
            target_thickness: NexusDataset::begin("target_thickness").finish(&dataset_register),
        }
    }
}
