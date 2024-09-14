use crate::{
    nexus::NexusSettings,
    schematic::{
        elements::{dataset::NexusDataset, group::NexusGroup, NexusBuildable, NexusGroupDef},
        groups::log::Log,
        nexus_class, H5String,
    },
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
