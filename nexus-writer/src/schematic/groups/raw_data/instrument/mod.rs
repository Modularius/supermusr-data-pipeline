use source::Source;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::schematic::{
    elements::{
        dataset::{Buildable, NexusDataset},
        group::{NexusGroup, NxGroup, NxPushMessage, RcGroupContentRegister, RcNexusGroup},
    },
    groups::log::Log,
    nexus_class, H5String,
};

mod source;

pub(super) struct Instrument {
    name: NexusDataset<H5String>,
    source: RcNexusGroup<Source>,
}

impl NxGroup for Instrument {
    const CLASS_NAME: &'static str = nexus_class::INSTRUMENT;

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            name: NexusDataset::begin().finish("name", dataset_register.clone()),
            source: NexusGroup::new("source", Some(dataset_register)),
        }
    }
}

impl<'a> NxPushMessage<RunStart<'a>> for Instrument {
    type MessageType = RunStart<'a>;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        Ok(())
    }
}
