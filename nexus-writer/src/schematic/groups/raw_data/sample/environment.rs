use hdf5::Group;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::{
    elements::{
        dataset::{NexusDataset, NexusDatasetMut},
        traits::{
            NexusDataHolderScalarMutable, NexusDataHolderStringMutable, NexusGroupDef,
            NexusHandleMessage,
        },
    },
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{nexus_class, H5String},
};

pub(super) struct Environment {
    /// name of apparatus
    name: NexusDatasetMut<H5String>,
    /// name displayed on DAE software
    short_name: NexusDatasetMut<H5String>,
    /// short code
    env_type: NexusDatasetMut<H5String>,
    /// long description
    description: NexusDatasetMut<H5String>,
    /// version of driver used to collect data, e.g. VI name and version
    program: NexusDatasetMut<H5String>,
    // /// log of hardware parameter relating to apparatus, e.g. temperature controller parameters, etc
    //hardware_log: NexusGroup<Log>,
    // /// log of hardware parameter relating to apparatus, e.g. temperature controller parameters, etc
    //sensor_name: NexusGroup<Sensor>,
}

impl NexusGroupDef for Environment {
    const CLASS_NAME: &'static str = nexus_class::ENVIRONMENT;
    type Settings = NexusSettings;

    fn new(_settings: &NexusSettings) -> Self {
        Self {
            name: NexusDataset::new_with_default("name"),
            short_name: NexusDataset::new_with_default("short_name"),
            env_type: NexusDataset::new_with_default("env_type"),
            description: NexusDataset::new_with_default("description"),
            program: NexusDataset::new_with_default("program"),
            //hardware_log: NexusGroup::new("hardware_log", settings),
            //sensor_name: NexusGroup::new("sensor_name", settings),
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>> for Environment {
    fn handle_message(
        &mut self,
        _message: &RunStart<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        self.name.write_string(parent, "Environment Name")?;
        self.short_name.write_string(parent, "Environment Name")?;
        self.env_type.write_string(parent, "Environment Name")?;
        self.description.write_string(parent, "Environment Name")?;
        self.program.write_string(parent, "Environment Name")?;
        Ok(())
    }
}
