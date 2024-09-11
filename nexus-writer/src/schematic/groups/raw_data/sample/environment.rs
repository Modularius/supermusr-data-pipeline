use crate::schematic::{
    elements::{
        dataset::NexusDataset, group::NexusGroup, NexusBuildable, NexusBuilderFinished,
        NexusGroupDef,
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

impl NexusGroupDef for Environment {
    const CLASS_NAME: &'static str = nexus_class::ENVIRONMENT;

    fn new() -> Self {
        Self {
            name: NexusDataset::begin("name")
                .default_value(Default::default())
                .finish(),
            short_name: NexusDataset::begin("short_name")
                .default_value(Default::default())
                .finish(),
            env_type: NexusDataset::begin("env_type")
                .default_value(Default::default())
                .finish(),
            description: NexusDataset::begin("description")
                .default_value(Default::default())
                .finish(),
            program: NexusDataset::begin("program")
                .default_value(Default::default())
                .finish(),
            hardware_log: NexusGroup::new("hardware_log"),
        }
    }
}
