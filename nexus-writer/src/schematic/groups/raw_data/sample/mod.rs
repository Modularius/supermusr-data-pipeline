use environment::Environment;
use geometry::Geometry;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::schematic::{
    elements::{
        dataset::NexusDataset, NexusBuildable, NexusBuilderFinished, NexusError, NexusGroupDef,
        NexusPushMessage,
    },
    nexus_class, H5String,
};

mod environment;
mod geometry;

pub(super) struct Sample {
    name: NexusDataset<H5String>,
    chemical_formula: NexusDataset<H5String>,
    description: NexusDataset<H5String>,
    sample_type: NexusDataset<H5String>,
    situation: NexusDataset<H5String>,
    shape: NexusDataset<H5String>,
    preparation_date: NexusDataset<H5String>,
    sample_holder: NexusDataset<H5String>,
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

    fn new() -> Self {
        Self {
            name: NexusDataset::begin("name")
                .default_value(Default::default())
                .finish(),
            chemical_formula: NexusDataset::begin("chemical_formula")
                .default_value(Default::default())
                .finish(),
            description: NexusDataset::begin("description")
                .default_value(Default::default())
                .finish(),
            sample_type: NexusDataset::begin("sample_type")
                .default_value(Default::default())
                .finish(),
            situation: NexusDataset::begin("situation")
                .default_value(Default::default())
                .finish(),
            shape: NexusDataset::begin("shape")
                .default_value(Default::default())
                .finish(),
            preparation_date: NexusDataset::begin("preparation_date")
                .default_value(Default::default())
                .finish(),
            sample_holder: NexusDataset::begin("sample_holder")
                .default_value(Default::default())
                .finish(),
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

impl<'a> NexusPushMessage<RunStart<'a>> for Sample {
    type MessageType = RunStart<'a>;

    fn push_message(&self, message: &Self::MessageType) -> Result<(), NexusError> {
        Ok(())
    }
}
