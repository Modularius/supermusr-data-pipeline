use hdf5::{Group, Location};
use source::Source;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::schematic::{
    elements::{
        dataset::NexusDataset, group::NexusGroup, NexusBuildable, NexusBuilderFinished, NexusError, NexusGroupDef, NexusHandleMessage, NexusPushMessage
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

impl<'a> NexusHandleMessage<RunStart<'a>> for Instrument {
    fn handle_message(&mut self, message: &RunStart<'a>, location: &Group) -> Result<(), NexusError> {
        Ok(())
    }
}
