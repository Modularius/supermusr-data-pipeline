use hdf5::Group;
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::{
    elements::{
        attribute::{NexusAttribute, NexusAttributeMut},
        dataset::{NexusDataset, NexusDatasetMut},
        group::NexusGroup,
        traits::{
            NexusDataHolderScalarMutable, NexusDatasetDef, NexusGroupDef, NexusHandleMessage,
        },
    },
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{groups::log::Log, nexus_class, H5String},
};

#[derive(Clone)]
struct FramesRequestedAttributes {
    _frame_type: NexusAttributeMut<H5String>,
}

impl NexusDatasetDef for FramesRequestedAttributes {
    fn new() -> Self {
        Self {
            _frame_type: NexusAttribute::new_with_default("frame_type"),
        }
    }
}

#[derive(Clone)]
struct LabelsAttributes {
    _separator: NexusAttributeMut<H5String>,
}

impl NexusDatasetDef for LabelsAttributes {
    fn new() -> Self {
        Self {
            _separator: NexusAttribute::new_with_default("separator"),
        }
    }
}

pub(super) struct Periods {
    _number: NexusDatasetMut<u32>,
    _period_types: NexusDatasetMut<u32>,
    _frames_requested: NexusDatasetMut<u32, FramesRequestedAttributes>,
    _output: NexusDatasetMut<u32>,
    _labels: NexusDatasetMut<H5String, LabelsAttributes>,
    _raw_frames: NexusDatasetMut<u32>,
    _good_frames: NexusDatasetMut<u32>,
    _sequences: NexusDatasetMut<u32>,
    _counts: NexusGroup<Log>,
}

impl NexusGroupDef for Periods {
    const CLASS_NAME: &'static str = nexus_class::PERIOD;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            _number: NexusDataset::new_with_default("number"),
            _period_types: NexusDataset::new_with_default("type"),
            _frames_requested: NexusDataset::new_with_default("frames_requested"),
            _output: NexusDataset::new_with_default("output"),
            _labels: NexusDataset::new_with_default("labels"),
            _raw_frames: NexusDataset::new_with_default("raw_frames"),
            _good_frames: NexusDataset::new_with_default("good_frames"),
            _sequences: NexusDataset::new_with_default("sequences"),
            _counts: NexusGroup::new("counts", settings),
        }
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>> for Periods {
    fn handle_message(
        &mut self,
        _message: &RunStart<'a>,
        _location: &Group,
    ) -> Result<(), NexusPushError> {
        Ok(())
    }
}
