//use environment::Environment;
//use geometry::Geometry;
use hdf5::Group;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::{
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{
        elements::{
            dataset::{NexusDataset, NexusDatasetMut},
            traits::{NexusBuildable, NexusGroupDef, NexusHandleMessage},
        },
        nexus_class, H5String,
    },
};

mod environment;
mod geometry;

pub(super) struct Sample {
    name: NexusDatasetMut<H5String>,
    chemical_formula: NexusDatasetMut<H5String>,
    description: NexusDatasetMut<H5String>,
    sample_type: NexusDatasetMut<H5String>,
    situation: NexusDatasetMut<H5String>,
    shape: NexusDatasetMut<H5String>,
    preparation_date: NexusDatasetMut<H5String>,
    sample_holder: NexusDatasetMut<H5String>,
    /*flypast: RcNexusDatasetVar<H5String>,
    geometry: NexusGroup<Geometry>,
    sample_component: RcNexusDatasetVar<H5String>,
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

impl NexusGroupDef for Sample {
    const CLASS_NAME: &'static str = nexus_class::SAMPLE;
    type Settings = NexusSettings;

    fn new(_settings: &NexusSettings) -> Self {
        Self {
            name: NexusDataset::begin("name").finish_with_auto_default(),
            chemical_formula: NexusDataset::begin("chemical_formula").finish_with_auto_default(),
            description: NexusDataset::begin("description").finish_with_auto_default(),
            sample_type: NexusDataset::begin("sample_type").finish_with_auto_default(),
            situation: NexusDataset::begin("situation").finish_with_auto_default(),
            shape: NexusDataset::begin("shape").finish_with_auto_default(),
            preparation_date: NexusDataset::begin("preparation_date").finish_with_auto_default(),
            sample_holder: NexusDataset::begin("sample_holder").finish_with_auto_default(),
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
        message: &RunStart<'a>,
        location: &Group,
    ) -> Result<(), NexusPushError> {
        Ok(())
    }
}
