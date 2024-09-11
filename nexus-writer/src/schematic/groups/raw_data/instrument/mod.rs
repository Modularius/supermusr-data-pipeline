use hdf5::Location;
use source::Source;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::schematic::{
    elements::{
        dataset::NexusDataset, group::NexusGroup, NexusBuildable, NexusBuilderFinished, NexusError,
        NexusGroupDef, NexusPushMessage,
    },
    groups::log::Log,
    nexus_class, H5String,
};

mod source;

pub(super) struct Instrument {
    name: NexusDataset<H5String>,
    source: NexusGroup<Source>,
}

impl NexusGroupDef for Instrument {
    const CLASS_NAME: &'static str = nexus_class::INSTRUMENT;

    fn new() -> Self {
        Self {
            name: NexusDataset::begin("name")
                .default_value(Default::default())
                .finish(),
            source: NexusGroup::new("source"),
        }
    }
}

impl<'a> NexusPushMessage<RunStart<'a>> for Instrument {
    type MessageType = RunStart<'a>;

    fn push_message(&self, message: &Self::MessageType, location: &Location) -> Result<(), NexusError> {
        Ok(())
    }
}
