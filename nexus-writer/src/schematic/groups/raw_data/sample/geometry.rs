use hdf5::Group;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::{
    elements::{
        dataset::{NexusDataset, NexusDatasetMut},
        traits::{NexusDataHolderScalarMutable, NexusDataHolderStringMutable, NexusGroupDef, NexusHandleMessage},
    }, error::NexusPushError, nexus::NexusSettings, schematic::{nexus_class, H5String}
};

pub(super) struct Geometry {
    name: NexusDatasetMut<H5String>,
}

impl NexusGroupDef for Geometry {
    const CLASS_NAME: &'static str = nexus_class::GEOMETRY;
    type Settings = NexusSettings;

    fn new(_settings: &NexusSettings) -> Self {
        Self {
            name: NexusDataset::new_with_default("name"),
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>> for Geometry {
    fn handle_message(
        &mut self,
        message: &RunStart<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        self.name.write_string(parent, "Geometry Name")?;
        Ok(())
    }
}