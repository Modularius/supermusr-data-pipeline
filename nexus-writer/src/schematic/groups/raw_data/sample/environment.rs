use hdf5::{types::VarLenAscii, Group};

use crate::schematic::{
    elements::{
        dataset::{NexusDataset, RcNexusDatasetVar},
        group::{NexusGroup, NxGroup, RcGroupContentRegister, RcNexusGroup},
    },
    groups::log::Log,
    nexus_class,
};

pub(super) struct Environment {
    name: RcNexusDatasetVar<VarLenAscii>,
    short_name: RcNexusDatasetVar<VarLenAscii>,
    env_type: RcNexusDatasetVar<VarLenAscii>,
    description: RcNexusDatasetVar<VarLenAscii>,
    program: RcNexusDatasetVar<VarLenAscii>,
    hardware_log: RcNexusGroup<Log>,
}

impl NxGroup for Environment {
    const CLASS_NAME: &'static str = nexus_class::ENVIRONMENT;

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            name: NexusDataset::begin().finish("name", dataset_register.clone()),
            short_name: NexusDataset::begin().finish("short_name", dataset_register.clone()),
            env_type: NexusDataset::begin().finish("env_type", dataset_register.clone()),
            description: NexusDataset::begin().finish("description", dataset_register.clone()),
            program: NexusDataset::begin().finish("program", dataset_register.clone()),
            hardware_log: NexusGroup::new("hardware_log", None),
        }
    }
}
