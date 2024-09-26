use crate::{
    elements::{
        dataset::{NexusDataset, NexusDatasetMut},
        group::NexusGroup,
        traits::{NexusDataHolderScalarMutable, NexusGroupDef},
    },
    nexus::NexusSettings,
    schematic::{groups::log::Log, nexus_class, H5String},
};

pub(super) struct Source {
    name: NexusDatasetMut<H5String>,
    source_type: NexusDatasetMut<H5String>,
    probe: NexusDatasetMut<H5String>,
    source_frequency: NexusDatasetMut<H5String>,
    source_frame_pattern: NexusDatasetMut<H5String>,
    source_energy: NexusDatasetMut<H5String>,
    source_current: NexusDatasetMut<H5String>,
    source_current_log: NexusGroup<Log>,
    source_pulse_width: NexusDatasetMut<H5String>,
    target_material: NexusDatasetMut<H5String>,
    target_thickness: NexusDatasetMut<H5String>,
}

impl NexusGroupDef for Source {
    const CLASS_NAME: &'static str = nexus_class::SOURCE;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            name: NexusDataset::new_with_default("name"),
            source_type: NexusDataset::new_with_default("source_type"),
            probe: NexusDataset::new_with_default("probe"),
            source_frequency: NexusDataset::new_with_default("source_frequency"),
            source_frame_pattern: NexusDataset::new_with_default("source_frame_pattern"),
            source_energy: NexusDataset::new_with_default("source_energy"),
            source_current: NexusDataset::new_with_default("tarsource_currentget_thickness"),
            source_current_log: NexusGroup::new("source_current_log", settings),
            source_pulse_width: NexusDataset::new_with_default("source_pulse_width"),
            target_material: NexusDataset::new_with_default("target_material"),
            target_thickness: NexusDataset::new_with_default("target_thickness"),
        }
    }
}
