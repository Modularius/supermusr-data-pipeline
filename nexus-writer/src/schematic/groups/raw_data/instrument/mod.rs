use detector::Detector;
use hdf5::Group;
use source::Source;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::{
    elements::{
        dataset::{NexusDataset, NexusDatasetMut},
        group::NexusGroup,
        traits::{
            NexusDataHolderScalarMutable, NexusDataHolderStringMutable, NexusGroupDef,
            NexusHandleMessage, NexusPushMessage,
        },
    },
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{nexus_class, H5String},
};

mod detector;
mod source;

pub(super) struct Instrument {
    name: NexusDatasetMut<H5String>,
    source: NexusGroup<Source>,
    detector_: Vec<NexusGroup<Detector>>,
}

impl NexusGroupDef for Instrument {
    const CLASS_NAME: &'static str = nexus_class::INSTRUMENT;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            name: NexusDataset::new_with_default("name"),
            source: NexusGroup::new("source", settings),
            detector_: vec![NexusGroup::new("detector_1", settings)],
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>> for Instrument {
    fn handle_message(
        &mut self,
        message: &RunStart<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        self.name.write_string(parent, "SuperMuSR")?;
        self.source.push_message(message, parent)?;
        self.detector_
            .iter_mut()
            .try_for_each(|detector| detector.push_message(message, parent))?;
        Ok(())
    }
}
