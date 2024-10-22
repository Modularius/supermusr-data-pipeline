use hdf5::Group;
use supermusr_common::Channel;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::{
    elements::{
        dataset::{NexusDataset, NexusDatasetResize},
        traits::{NexusAppendableDataHolder, NexusGroupDef, NexusHandleMessage},
    },
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::nexus_class,
};

pub(super) struct Detector {
    /// 1, 2 or 3D array of counts, [period, spectrum, time bin]
    counts: NexusDatasetResize<usize>,
    /// list of global spectra
    spectrum_index: NexusDatasetResize<Channel>,
}

impl NexusGroupDef for Detector {
    const CLASS_NAME: &'static str = nexus_class::DETECTOR;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            counts: NexusDataset::new_appendable_with_default(
                "counts",
                settings.eventlist_chunk_size,
            ),
            spectrum_index: NexusDataset::new_appendable_with_default(
                "spectrum_index",
                settings.framelist_chunk_size,
            ),
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>> for Detector {
    fn handle_message(
        &mut self,
        _message: &RunStart<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        self.counts.append(parent, &[0])?;
        self.spectrum_index.append(parent, &[0])?;
        Ok(())
    }
}
