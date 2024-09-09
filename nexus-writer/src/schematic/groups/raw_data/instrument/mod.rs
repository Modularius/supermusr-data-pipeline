use source::Source;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::schematic::{
    elements::{
        dataset::NexusDataset,
        group::{GroupContentRegister, NexusGroup, NxGroup, NxPushMessage},
        traits::{Buildable, SubgroupBuildable},
    },
    groups::log::Log,
    nexus_class, H5String,
};

mod source;

pub(super) struct Instrument {
    name: NexusDataset<H5String>,
    source: NexusGroup<Source>,
}

impl NxGroup for Instrument {
    const CLASS_NAME: &'static str = nexus_class::INSTRUMENT;

    fn new(dataset_register: GroupContentRegister) -> Self {
        Self {
            name: NexusDataset::begin("name")
                .default_value(Default::default())
                .finish(&dataset_register),
            source: NexusGroup::new_subgroup("source", &dataset_register),
        }
    }
}

impl<'a> NxPushMessage<RunStart<'a>> for Instrument {
    type MessageType = RunStart<'a>;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        Ok(())
    }
}
