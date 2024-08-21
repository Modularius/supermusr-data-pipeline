use hdf5::{types::VarLenAscii, Group};

use crate::schematic::{
    elements::{
        dataset::NexusDataset,
        group::{NexusGroup, NxGroup},
    },
    groups::log::Log,
};

pub(super) struct Environment {
    name: NexusDataset<VarLenAscii>,
    short_name: NexusDataset<VarLenAscii>,
    env_type: NexusDataset<VarLenAscii>,
    description: NexusDataset<VarLenAscii>,
    program: NexusDataset<VarLenAscii>,
    hardware_log: NexusGroup<Log>,
}

impl NxGroup for Environment {
    const CLASS_NAME: &'static str = "NXenvironment";

    fn new() -> Self {
        Self {
            name: NexusDataset::begin().finish("name"),
            short_name: NexusDataset::begin().finish("short_name"),
            env_type: NexusDataset::begin().finish("env_type"),
            description: NexusDataset::begin().finish("description"),
            program: NexusDataset::begin().finish("program"),
            hardware_log: NexusGroup::new("hardware_log"),
        }
    }

    fn create(&mut self, this: &Group) {
        self.name.create(this);
        self.short_name.create(this);
        self.env_type.create(this);
        self.description.create(this);
        self.program.create(this);
        self.hardware_log.create(this);
    }

    fn open(&mut self, this: &Group) {
        self.name.open(this);
        self.short_name.open(this);
        self.env_type.open(this);
        self.description.open(this);
        self.program.open(this);
        self.hardware_log.open(this);
    }

    fn close(&mut self) {
        self.name.close();
        self.short_name.close();
        self.env_type.close();
        self.description.close();
        self.program.close();
        self.hardware_log.close();
    }
}
