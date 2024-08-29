use crate::schematic::{
    elements::{
        dataset::NexusDataset,traits::Buildable,
        group::{NexusGroup, NxGroup, RcGroupContentRegister, RcNexusGroup},
    },
    groups::log::Log,
    nexus_class, H5String,
};

pub(super) struct Environment {
    name: NexusDataset<H5String>,
    short_name: NexusDataset<H5String>,
    env_type: NexusDataset<H5String>,
    description: NexusDataset<H5String>,
    program: NexusDataset<H5String>,
    hardware_log: RcNexusGroup<Log>,
}

impl NxGroup for Environment {
    const CLASS_NAME: &'static str = nexus_class::ENVIRONMENT;

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            name: NexusDataset::begin("name").finish(&dataset_register),
            short_name: NexusDataset::begin("short_name").finish(&dataset_register),
            env_type: NexusDataset::begin("env_type").finish(&dataset_register),
            description: NexusDataset::begin("description").finish(&dataset_register),
            program: NexusDataset::begin("program").finish(&dataset_register),
            hardware_log: NexusGroup::new("hardware_log", &dataset_register),
        }
    }
}
