use crate::schematic::{
    elements::{
        dataset::NexusDataset,
        group::{GroupContentRegister, NexusGroup, NxGroup},
        traits::{Buildable, SubgroupBuildable},
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
    hardware_log: NexusGroup<Log>,
}

impl NxGroup for Environment {
    const CLASS_NAME: &'static str = nexus_class::ENVIRONMENT;

    fn new(dataset_register: GroupContentRegister) -> Self {
        Self {
            name: NexusDataset::begin("name")
                .default_value(Default::default())
                .finish(&dataset_register),
            short_name: NexusDataset::begin("short_name")
                .default_value(Default::default())
                .finish(&dataset_register),
            env_type: NexusDataset::begin("env_type")
                .default_value(Default::default())
                .finish(&dataset_register),
            description: NexusDataset::begin("description")
                .default_value(Default::default())
                .finish(&dataset_register),
            program: NexusDataset::begin("program")
                .default_value(Default::default())
                .finish(&dataset_register),
            hardware_log: NexusGroup::new_subgroup("hardware_log", &dataset_register),
        }
    }
}
