use supermusr_common::Channel;

use crate::{
    elements::{
        dataset::{NexusDataset, NexusDatasetResize},
        traits::{NexusAppendableDataHolder, NexusGroupDef},
    },
    nexus::NexusSettings,
    schematic::nexus_class,
};

pub(super) struct Detector {
    _counts: NexusDatasetResize<usize>,
    _spectrum_index: NexusDatasetResize<Channel>,
}

impl NexusGroupDef for Detector {
    const CLASS_NAME: &'static str = nexus_class::DETECTOR;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            _counts: NexusDataset::new_appendable_with_default("counts", settings.eventlist_chunk_size),
            _spectrum_index: NexusDataset::new_appendable_with_default("spectrum_index", settings.framelist_chunk_size),
        }
    }
}
