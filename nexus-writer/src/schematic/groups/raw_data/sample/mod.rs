use environment::Environment;
use geometry::Geometry;
use hdf5::{types::VarLenAscii, Group};
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::schematic::{
    elements::{
        dataset::NexusDataset,
        group::{NexusGroup, NxGroup, NxPushMessage},
    },
    groups::log::Log,
};

mod environment;
mod geometry;

pub(super) struct Sample {
    name: NexusDataset<VarLenAscii>,
    chemical_formula: NexusDataset<VarLenAscii>,
    description: NexusDataset<VarLenAscii>,
    sample_type: NexusDataset<VarLenAscii>,
    situation: NexusDataset<VarLenAscii>,
    shape: NexusDataset<VarLenAscii>,
    preparation_date: NexusDataset<VarLenAscii>,
    sample_holder: NexusDataset<VarLenAscii>,
    /*flypast: NexusDataset<VarLenAscii>,
    geometry: NexusGroup<Geometry>,
    sample_component: NexusDataset<VarLenAscii>,
    thickness: NexusDataset<u32, MustEnterAttributes<1>>,
    mass: NexusDataset<u32, MustEnterAttributes<1>>,
    density: NexusDataset<u32, MustEnterAttributes<1>>,
    temperature: NexusDataset<u32>,
    magnetic_field: NexusDataset<u32>,
    magnetic_field_state: NexusDataset<u32>,
    temperature_: NexusDataset<u32, MustEnterAttributes<3>>,
    temperature__env: NexusGroup<Environment>,
    temperature__log: NexusGroup<Log>,
    magnetic_field_: NexusDataset<u32, MustEnterAttributes<3>>,
    magnetic_field__env: NexusGroup<Environment>,
    magnetic_field__log: NexusGroup<Log>,*/
}

impl NxGroup for Sample {
    const CLASS_NAME: &'static str = "NXperiod";

    fn new() -> Self {
        Self {
            name: NexusDataset::begin().finish("name"),
            chemical_formula: NexusDataset::begin().finish("chemical_formula"),
            description: NexusDataset::begin().finish("description"),
            sample_type: NexusDataset::begin().finish("sample_type"),
            situation: NexusDataset::begin().finish("situation"),
            shape: NexusDataset::begin().finish("shape"),
            preparation_date: NexusDataset::begin().finish("preparation_date"),
            sample_holder: NexusDataset::begin().finish("sample_holder"),
            /*flypast: NexusDataset::begin().finish("flypast"),
            geometry: NexusGroup::new("geometry"),
            sample_component: NexusDataset::begin().finish("sample_component"),
            thickness: NexusDataset::begin().finish("thickness"),
            mass: NexusDataset::begin().finish("mass"),
            density: NexusDataset::begin().finish("density"),
            temperature: NexusDataset::begin().finish("temperature"),
            magnetic_field: NexusDataset::begin().finish("magnetic_field"),
            magnetic_field_state: NexusDataset::begin().finish("magnetic_field_state"),
            temperature_: NexusDataset::begin().finish("temperature_1"),
            temperature__env: NexusGroup::new("temperature_1_env"),
            temperature__log: NexusGroup::new("temperature_1_log"),
            magnetic_field_: NexusDataset::begin().finish("magnetic_field_1"),
            magnetic_field__env: NexusGroup::new("magnetic_field_1_env"),
            magnetic_field__log: NexusGroup::new("magnetic_field_1_log"),*/
        }
    }

    fn create(&mut self, this: &Group) {
        self.name.create(this);
        self.chemical_formula.create(this);
        self.description.create(this);
        self.sample_type.create(this);
        self.situation.create(this);
        self.shape.create(this);
        self.preparation_date.create(this);
        self.sample_holder.create(this);
    }

    fn open(&mut self, this: &Group) {
        self.name.open(this);
        self.chemical_formula.open(this);
        self.description.open(this);
        self.sample_type.open(this);
        self.situation.open(this);
        self.shape.open(this);
        self.preparation_date.open(this);
        self.sample_holder.open(this);
    }

    fn close(&mut self) {
        self.name.close();
        self.chemical_formula.close();
        self.description.close();
        self.sample_type.close();
        self.situation.close();
        self.shape.close();
        self.preparation_date.close();
        self.sample_holder.close();
    }
}


impl<'a> NxPushMessage<RunStart<'a>> for Sample {
    type MessageType = RunStart<'a>;
    
    fn push_message(&mut self, message: &Self::MessageType) {
        todo!()
    }
}