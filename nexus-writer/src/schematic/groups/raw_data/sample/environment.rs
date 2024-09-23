use hdf5::types::TypeDescriptor;

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
            name: NexusDataset::begin("name").finish_with_auto_default(),
            short_name: NexusDataset::begin("short_name").finish_with_auto_default(),
            env_type: NexusDataset::begin("env_type").finish_with_auto_default(),
            description: NexusDataset::begin("description").finish_with_auto_default(),
            program: NexusDataset::begin("program").finish_with_auto_default(),
            hardware_log: NexusGroup::new(
                "hardware_log",
                &(settings.clone(), TypeDescriptor::VarLenUnicode),
            ),
        }
    }
}
