use environment::Environment;
use geometry::Geometry;
use hdf5::types::VarLenAscii;

use crate::schematic::{elements::{dataset::NexusDataset, NexusGroup, NxGroup}, groups::log::Log};

mod geometry;
mod environment;

pub(super) struct Sample {
    name: NexusDataset<VarLenAscii>,
    chemical_formula: NexusDataset<VarLenAscii>,
    description: NexusDataset<VarLenAscii>,
    sample_type: NexusDataset<VarLenAscii>,
    situation: NexusDataset<VarLenAscii>,
    shape: NexusDataset<VarLenAscii>,
    preparation_date: NexusDataset<VarLenAscii>,
    sample_holder: NexusDataset<VarLenAscii>,
    flypast: NexusDataset<VarLenAscii>,
    geometry: NexusGroup<Geometry>,
    sample_component: NexusDataset<VarLenAscii>,
    thickness: NexusDataset<u32>,
    mass: NexusDataset<u32>,
    density: NexusDataset<u32>,
    temperature: NexusDataset<u32>,
    magnetic_field: NexusDataset<u32>,
    magnetic_field_state: NexusDataset<u32>,
    temperature_: NexusDataset<u32>,
    temperature__env: NexusGroup<Environment>,
    temperature__log: NexusGroup<Log>,
    magnetic_field_: NexusDataset<u32>,
    magnetic_field__env: NexusGroup<Environment>,
    magnetic_field__log: NexusGroup<Log>,
}

impl NxGroup for Sample {
    const CLASS_NAME : &'static str = "NXperiod";

    fn new() -> Self {
        Self {
            name: NexusDataset::new("name"),
            chemical_formula: NexusDataset::new("chemical_formula"),
            description: NexusDataset::new("description"),
            sample_type: NexusDataset::new("sample_type"),
            situation: NexusDataset::new("situation"),
            shape: NexusDataset::new("shape"),
            preparation_date: NexusDataset::new("preparation_date"),
            sample_holder: NexusDataset::new("sample_holder"),
            flypast: NexusDataset::new("flypast"),
            geometry: NexusGroup::new("geometry"),
            sample_component: NexusDataset::new("sample_component"),
            thickness: NexusDataset::new("thickness"),
            mass: NexusDataset::new("mass"),
            density: NexusDataset::new("density"),
            temperature: NexusDataset::new("temperature"),
            magnetic_field: NexusDataset::new("magnetic_field"),
            magnetic_field_state: NexusDataset::new("magnetic_field_state"),
            temperature_: NexusDataset::new("temperature_1"),
            temperature__env: NexusGroup::new("temperature_1_env"),
            temperature__log: NexusGroup::new("temperature_1_log"),
            magnetic_field_: NexusDataset::new("magnetic_field_1"),
            magnetic_field__env: NexusGroup::new("magnetic_field_1_env"),
            magnetic_field__log: NexusGroup::new("magnetic_field_1_log"),
        }
    }
}