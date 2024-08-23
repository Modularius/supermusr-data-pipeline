use environment::Environment;
use geometry::Geometry;
use hdf5::types::VarLenAscii;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::schematic::elements::{
    dataset::{NexusDataset, RcNexusDatasetVar},
    group::{NxGroup, NxPushMessage, RcDatasetRegister},
};

mod environment;
mod geometry;

pub(super) struct Sample {
    name: RcNexusDatasetVar<VarLenAscii>,
    chemical_formula: RcNexusDatasetVar<VarLenAscii>,
    description: RcNexusDatasetVar<VarLenAscii>,
    sample_type: RcNexusDatasetVar<VarLenAscii>,
    situation: RcNexusDatasetVar<VarLenAscii>,
    shape: RcNexusDatasetVar<VarLenAscii>,
    preparation_date: RcNexusDatasetVar<VarLenAscii>,
    sample_holder: RcNexusDatasetVar<VarLenAscii>,
    /*flypast: RcNexusDatasetVar<VarLenAscii>,
    geometry: NexusGroup<Geometry>,
    sample_component: RcNexusDatasetVar<VarLenAscii>,
    thickness: RcNexusDatasetVar<u32, MustEnterAttributes<1>>,
    mass: RcNexusDatasetVar<u32, MustEnterAttributes<1>>,
    density: RcNexusDatasetVar<u32, MustEnterAttributes<1>>,
    temperature: RcNexusDatasetVar<u32>,
    magnetic_field: RcNexusDatasetVar<u32>,
    magnetic_field_state: RcNexusDatasetVar<u32>,
    temperature_: RcNexusDatasetVar<u32, MustEnterAttributes<3>>,
    temperature__env: NexusGroup<Environment>,
    temperature__log: NexusGroup<Log>,
    magnetic_field_: RcNexusDatasetVar<u32, MustEnterAttributes<3>>,
    magnetic_field__env: NexusGroup<Environment>,
    magnetic_field__log: NexusGroup<Log>,*/
}

impl NxGroup for Sample {
    const CLASS_NAME: &'static str = "NXperiod";

    fn new(dataset_register : RcDatasetRegister) -> Self {
        Self {
            name: NexusDataset::begin().finish("name", dataset_register.clone()),
            chemical_formula: NexusDataset::begin().finish("chemical_formula", dataset_register.clone()),
            description: NexusDataset::begin().finish("description", dataset_register.clone()),
            sample_type: NexusDataset::begin().finish("sample_type", dataset_register.clone()),
            situation: NexusDataset::begin().finish("situation", dataset_register.clone()),
            shape: NexusDataset::begin().finish("shape", dataset_register.clone()),
            preparation_date: NexusDataset::begin().finish("preparation_date", dataset_register.clone()),
            sample_holder: NexusDataset::begin().finish("sample_holder", dataset_register),
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
}


impl<'a> NxPushMessage<RunStart<'a>> for Sample {
    type MessageType = RunStart<'a>;
    
    fn push_message(&mut self, message: &Self::MessageType) {
        todo!()
    }
}