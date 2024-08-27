use hdf5::{types::VarLenAscii, Group};
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::schematic::{
    elements::{
        dataset::{NexusDataset, RcNexusDatasetVar},
        group::{NexusGroup, NxGroup, NxPushMessage, RcGroupContentRegister, RcNexusGroup},
    },
    groups::log::Log, nexus_class,
};

pub(super) struct Source {
    name: RcNexusDatasetVar<VarLenAscii>,
    source_type: RcNexusDatasetVar<VarLenAscii>,
    probe: RcNexusDatasetVar<VarLenAscii>,
    source_frequency: RcNexusDatasetVar<VarLenAscii>,
    source_frame_pattern: RcNexusDatasetVar<VarLenAscii>,
    source_energy: RcNexusDatasetVar<VarLenAscii>,
    source_current: RcNexusDatasetVar<VarLenAscii>,
    source_current_log: RcNexusGroup<Log>,
    source_pulse_width: RcNexusDatasetVar<VarLenAscii>,
    target_material: RcNexusDatasetVar<VarLenAscii>,
    target_thickness: RcNexusDatasetVar<VarLenAscii>,
}

impl NxGroup for Source {
    const CLASS_NAME: &'static str = nexus_class::SOURCE;

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            name: NexusDataset::begin().finish("name", dataset_register.clone()),
            source_type: NexusDataset::begin().finish("source_type", dataset_register.clone()),
            probe: NexusDataset::begin().finish("probe", dataset_register.clone()),
            source_frequency: NexusDataset::begin()
                .finish("source_frequency", dataset_register.clone()),
            source_frame_pattern: NexusDataset::begin()
                .finish("source_frame_pattern", dataset_register.clone()),
            source_energy: NexusDataset::begin().finish("source_energy", dataset_register.clone()),
            source_current: NexusDataset::begin()
                .finish("source_current", dataset_register.clone()),
            source_current_log: NexusGroup::new("source_current_log", None),
            source_pulse_width: NexusDataset::begin()
                .finish("source_pulse_width", dataset_register.clone()),
            target_material: NexusDataset::begin()
                .finish("target_material", dataset_register.clone()),
            target_thickness: NexusDataset::begin()
                .finish("target_thickness", dataset_register.clone()),
        }
    }
}