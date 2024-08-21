use hdf5::{types::VarLenAscii, Group};
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::schematic::{
    elements::{
        dataset::NexusDataset,
        group::{NexusGroup, NxGroup, NxPushMessage},
    },
    groups::log::Log,
};

pub(super) struct Instrument {
    name: NexusDataset<VarLenAscii>,
    source: NexusGroup<Source>,
}

impl NxGroup for Instrument {
    const CLASS_NAME: &'static str = "NXinstrument";

    fn new() -> Self {
        Self {
            name: NexusDataset::begin().finish(""),
            source: NexusGroup::new("source"),
        }
    }

    fn create(&mut self, parent: &Group) {
        self.name.create(parent);
        self.source.create(parent);
    }

    fn open(&mut self, parent: &Group) {
        self.name.open(parent);
        self.source.open(parent);
    }

    fn close(&mut self) {
        self.name.close();
        self.source.close();
    }
}

pub(super) struct Source {
    name: NexusDataset<VarLenAscii>,
    source_type: NexusDataset<VarLenAscii>,
    probe: NexusDataset<VarLenAscii>,
    source_frequency: NexusDataset<VarLenAscii>,
    source_frame_pattern: NexusDataset<VarLenAscii>,
    source_energy: NexusDataset<VarLenAscii>,
    source_current: NexusDataset<VarLenAscii>,
    source_current_log: NexusGroup<Log>,
    source_pulse_width: NexusDataset<VarLenAscii>,
    target_material: NexusDataset<VarLenAscii>,
    target_thickness: NexusDataset<VarLenAscii>,
}

impl NxGroup for Source {
    const CLASS_NAME: &'static str = "NXsource";

    fn new() -> Self {
        Self {
            name: NexusDataset::begin().finish("name"),
            source_type: NexusDataset::begin().finish("source_type"),
            probe: NexusDataset::begin().finish("probe"),
            source_frequency: NexusDataset::begin().finish("source_frequency"),
            source_frame_pattern: NexusDataset::begin().finish("source_frame_pattern"),
            source_energy: NexusDataset::begin().finish("source_energy"),
            source_current: NexusDataset::begin().finish("source_current"),
            source_current_log: NexusGroup::new("source_current_log"),
            source_pulse_width: NexusDataset::begin().finish("source_pulse_width"),
            target_material: NexusDataset::begin().finish("target_material"),
            target_thickness: NexusDataset::begin().finish("target_thickness"),
        }
    }

    fn create(&mut self, this: &Group) {
        self.name.create(this);
        self.source_type.create(this);
        self.probe.create(this);
        self.source_frequency.create(this);
        self.source_frame_pattern.create(this);
        self.probe.create(this);
        self.source_frequency.create(this);
        self.source_frame_pattern.create(this);
        self.source_energy.create(this);
        self.source_current.create(this);
        self.source_current_log.create(this);
        self.source_pulse_width.create(this);
        self.target_material.create(this);
        self.target_thickness.create(this);
    }

    fn open(&mut self, this: &Group) {
        self.name.open(this);
        self.source_type.open(this);
        self.probe.open(this);
        self.source_frequency.open(this);
        self.source_frame_pattern.open(this);
        self.probe.open(this);
        self.source_frequency.open(this);
        self.source_frame_pattern.open(this);
        self.source_energy.open(this);
        self.source_current.open(this);
        self.source_current_log.open(this);
        self.source_pulse_width.open(this);
        self.target_material.open(this);
        self.target_thickness.open(this);
    }

    fn close(&mut self) {
        self.name.close();
        self.source_type.close();
        self.probe.close();
        self.source_frequency.close();
        self.source_frame_pattern.close();
        self.probe.close();
        self.source_frequency.close();
        self.source_frame_pattern.close();
        self.source_energy.close();
        self.source_current.close();
        self.source_current_log.close();
        self.source_pulse_width.close();
        self.target_material.close();
        self.target_thickness.close();
    }
}


impl<'a> NxPushMessage<RunStart<'a>> for Instrument {
    type MessageType = RunStart<'a>;
    
    fn push_message(&mut self, message: &Self::MessageType) {
        todo!()
    }
}