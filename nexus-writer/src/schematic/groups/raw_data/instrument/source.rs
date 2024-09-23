use crate::{
    nexus::NexusSettings,
    schematic::{
        elements::{
            dataset::{NexusDataset, NexusDatasetMut},
            group::NexusGroup,
            NexusBuildable, NexusGroupDef,
        },
        groups::log::Log,
        nexus_class, H5String,
    },
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
            name: NexusDataset::begin("name").finish_with_auto_default(),
            source_type: NexusDataset::begin("source_type").finish_with_auto_default(),
            probe: NexusDataset::begin("probe").finish_with_auto_default(),
            source_frequency: NexusDataset::begin("source_frequency").finish_with_auto_default(),
            source_frame_pattern: NexusDataset::begin("source_frame_pattern")
                .finish_with_auto_default(),
            source_energy: NexusDataset::begin("source_energy").finish_with_auto_default(),
            source_current: NexusDataset::begin("tarsource_currentget_thickness")
                .finish_with_auto_default(),
            source_current_log: NexusGroup::new("source_current_log", settings),
            source_pulse_width: NexusDataset::begin("source_pulse_width")
                .finish_with_auto_default(),
            target_material: NexusDataset::begin("target_material").finish_with_auto_default(),
            target_thickness: NexusDataset::begin("target_thickness").finish_with_auto_default(),
        }
    }
}
