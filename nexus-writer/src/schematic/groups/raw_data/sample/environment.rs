use crate::{
    nexus::NexusSettings,
    schematic::{
        elements::{
            dataset::{NexusDataset, NexusDatasetMut},
            group::NexusGroup,
            traits::{NexusDataHolderScalarMutable, NexusGroupDef},
        },
        groups::log::Log,
        nexus_class, H5String,
    },
};

pub(super) struct Environment {
    name: NexusDatasetMut<H5String>,
    short_name: NexusDatasetMut<H5String>,
    env_type: NexusDatasetMut<H5String>,
    description: NexusDatasetMut<H5String>,
    program: NexusDatasetMut<H5String>,
    hardware_log: NexusGroup<Log>,
}

impl NexusGroupDef for Environment {
    const CLASS_NAME: &'static str = nexus_class::ENVIRONMENT;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            name: NexusDataset::new_with_auto_default("name"),
            short_name: NexusDataset::new_with_auto_default("short_name"),
            env_type: NexusDataset::new_with_auto_default("env_type"),
            description: NexusDataset::new_with_auto_default("description"),
            program: NexusDataset::new_with_auto_default("program"),
            hardware_log: NexusGroup::new("hardware_log", settings),
        }
    }
}
