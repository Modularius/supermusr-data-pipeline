use hdf5::Group;
use source::Source;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::{
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{
        elements::{
            dataset::{NexusDataset, NexusDatasetMut}, group::NexusGroup, NexusBuildable, NexusGroupDef,
            NexusHandleMessage,
        },
        nexus_class, H5String,
    },
};

mod source;

pub(super) struct Instrument {
    name: NexusDatasetMut<H5String>,
    source: NexusGroup<Source>,
}

impl NexusGroupDef for Instrument {
    const CLASS_NAME: &'static str = nexus_class::INSTRUMENT;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            name: NexusDataset::begin("name").finish_with_auto_default(),
            source: NexusGroup::new("source", settings),
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>> for Instrument {
    fn handle_message(
        &mut self,
        message: &RunStart<'a>,
        location: &Group,
    ) -> Result<(), NexusPushError> {
        Ok(())
    }
}
