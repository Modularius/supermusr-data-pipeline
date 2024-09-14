use crate::{
    nexus::NexusSettings,
    schematic::{
        elements::{dataset::NexusDataset, group::NexusGroup, NexusBuildable, NexusGroupDef},
        groups::log::Log,
        nexus_class, H5String,
    },
};

pub(super) struct Environment {
    name: NexusDataset<H5String>,
    short_name: NexusDataset<H5String>,
    env_type: NexusDataset<H5String>,
    description: NexusDataset<H5String>,
    program: NexusDataset<H5String>,
    hardware_log: NexusGroup<Log>,
}

impl NexusGroupDef for Environment {
    const CLASS_NAME: &'static str = nexus_class::ENVIRONMENT;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            name: NexusDataset::begin("name").finish_with_auto_default(),
            short_name: NexusDataset::begin("short_name").finish_with_auto_default(),
            env_type: NexusDataset::begin("env_type").finish_with_auto_default(),
            description: NexusDataset::begin("description").finish_with_auto_default(),
            program: NexusDataset::begin("program").finish_with_auto_default(),
            hardware_log: NexusGroup::new("hardware_log", settings),
        }
    }
}
