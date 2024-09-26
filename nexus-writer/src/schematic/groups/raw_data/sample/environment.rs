use crate::{
    elements::{
        dataset::{NexusDataset, NexusDatasetMut},
        group::NexusGroup,
        traits::{NexusDataHolderScalarMutable, NexusGroupDef},
    },
    nexus::NexusSettings,
    schematic::{groups::log::Log, nexus_class, H5String},
};

pub(super) struct Environment {
    _name: NexusDatasetMut<H5String>,
    _short_name: NexusDatasetMut<H5String>,
    _env_type: NexusDatasetMut<H5String>,
    _description: NexusDatasetMut<H5String>,
    _program: NexusDatasetMut<H5String>,
    _hardware_log: NexusGroup<Log>,
}

impl NexusGroupDef for Environment {
    const CLASS_NAME: &'static str = nexus_class::ENVIRONMENT;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            _name: NexusDataset::new_with_default("name"),
            _short_name: NexusDataset::new_with_default("short_name"),
            _env_type: NexusDataset::new_with_default("env_type"),
            _description: NexusDataset::new_with_default("description"),
            _program: NexusDataset::new_with_default("program"),
            _hardware_log: NexusGroup::new("hardware_log", settings),
        }
    }
}
