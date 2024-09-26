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
    _name: NexusDatasetMut<H5String>,
    _source_type: NexusDatasetMut<H5String>,
    _probe: NexusDatasetMut<H5String>,
    _source_frequency: NexusDatasetMut<H5String>,
    _source_frame_pattern: NexusDatasetMut<H5String>,
    _source_energy: NexusDatasetMut<H5String>,
    _source_current: NexusDatasetMut<H5String>,
    _source_current_log: NexusGroup<Log>,
    _source_pulse_width: NexusDatasetMut<H5String>,
    _target_material: NexusDatasetMut<H5String>,
    _target_thickness: NexusDatasetMut<H5String>,
}

impl NexusGroupDef for Source {
    const CLASS_NAME: &'static str = nexus_class::SOURCE;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            _name: NexusDataset::new_with_default("name"),
            _source_type: NexusDataset::new_with_default("source_type"),
            _probe: NexusDataset::new_with_default("probe"),
            _source_frequency: NexusDataset::new_with_default("source_frequency"),
            _source_frame_pattern: NexusDataset::new_with_default("source_frame_pattern"),
            _source_energy: NexusDataset::new_with_default("source_energy"),
            _source_current: NexusDataset::new_with_default("tarsource_currentget_thickness"),
            _source_current_log: NexusGroup::new("source_current_log", settings),
            _source_pulse_width: NexusDataset::new_with_default("source_pulse_width"),
            _target_material: NexusDataset::new_with_default("target_material"),
            _target_thickness: NexusDataset::new_with_default("target_thickness"),
        }
    }
}
