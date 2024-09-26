//use environment::Environment;
//use geometry::Geometry;
use hdf5::Group;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::{
    elements::{
        dataset::{NexusDataset, NexusDatasetMut},
        traits::{NexusDataHolderScalarMutable, NexusGroupDef, NexusHandleMessage},
    },
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{nexus_class, H5String},
};

mod environment;
mod geometry;

pub(super) struct Sample {
    _name: NexusDatasetMut<H5String>,
    _chemical_formula: NexusDatasetMut<H5String>,
    _description: NexusDatasetMut<H5String>,
    _sample_type: NexusDatasetMut<H5String>,
    _situation: NexusDatasetMut<H5String>,
    _shape: NexusDatasetMut<H5String>,
    _preparation_date: NexusDatasetMut<H5String>,
    _sample_holder: NexusDatasetMut<H5String>,
    /*flypast: NexusDatasetMut<H5String>,
    geometry: NexusGroup<Geometry>,
    sample_component: NexusDatasetMut<H5String>,
    thickness: NexusDatasetMut<u32, MustEnterAttributes<1>>,
    mass: NexusDatasetMut<u32, MustEnterAttributes<1>>,
    density: NexusDatasetMut<u32, MustEnterAttributes<1>>,
    temperature: NexusDatasetMut<u32>,
    magnetic_field: NexusDatasetMut<u32>,
    magnetic_field_state: NexusDatasetMut<u32>,
    temperature_: NexusDatasetMut<u32, MustEnterAttributes<3>>,
    temperature__env: NexusGroup<Environment>,
    temperature__log: NexusGroup<Log>,
    magnetic_field_: NexusDatasetMut<u32, MustEnterAttributes<3>>,
    magnetic_field__env: NexusGroup<Environment>,
    magnetic_field__log: NexusGroup<Log>,*/
}

impl NexusGroupDef for Sample {
    const CLASS_NAME: &'static str = nexus_class::SAMPLE;
    type Settings = NexusSettings;

    fn new(_settings: &NexusSettings) -> Self {
        Self {
            _name: NexusDataset::new_with_default("name"),
            _chemical_formula: NexusDataset::new_with_default("chemical_formula"),
            _description: NexusDataset::new_with_default("description"),
            _sample_type: NexusDataset::new_with_default("sample_type"),
            _situation: NexusDataset::new_with_default("situation"),
            _shape: NexusDataset::new_with_default("shape"),
            _preparation_date: NexusDataset::new_with_default("preparation_date"),
            _sample_holder: NexusDataset::new_with_default("sample_holder"),
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

impl<'a> NexusHandleMessage<RunStart<'a>> for Sample {
    fn handle_message(
        &mut self,
        _message: &RunStart<'a>,
        _location: &Group,
    ) -> Result<(), NexusPushError> {
        Ok(())
    }
}
