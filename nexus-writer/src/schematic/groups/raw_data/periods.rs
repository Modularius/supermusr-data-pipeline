use hdf5::types::{TypeDescriptor, VarLenAscii};
use supermusr_streaming_types::ecs_pl72_run_start_generated::RunStart;

use crate::schematic::{
    elements::{
        attribute::NexusAttribute,
        dataset::{MustEnterAttributes, NexusDataset},
        group::{NexusGroup, NxGroup, NxPushMessage},
    },
    groups::log::Log,
};

pub(super) struct Periods {
    number: NexusDataset<u32>,
    period_types: NexusDataset<u32>,
    frames_requested: NexusDataset<u32, MustEnterAttributes<1>>,
    output: NexusDataset<u32>,
    labels: NexusDataset<VarLenAscii, MustEnterAttributes<1>>,
    raw_frames: NexusDataset<u32>,
    good_frames: NexusDataset<u32>,
    sequences: NexusDataset<u32>,
    counts: NexusGroup<Log>,
}

impl NxGroup for Periods {
    const CLASS_NAME: &'static str = "NXperiod";

    fn new() -> Self {
        Self {
            number: NexusDataset::begin().finish("number"),
            period_types: NexusDataset::begin().finish("type"),
            frames_requested: NexusDataset::begin()
                .attributes([NexusAttribute::new(
                    "frame_type",
                    TypeDescriptor::VarLenAscii,
                )])
                .finish("frames_requested"),
            output: NexusDataset::begin().finish("output"),
            labels: NexusDataset::begin()
                .attributes([NexusAttribute::new(
                    "separator",
                    TypeDescriptor::VarLenAscii,
                )])
                .finish("labels"),
            raw_frames: NexusDataset::begin().finish("raw_frames"),
            good_frames: NexusDataset::begin().finish("good_frames"),
            sequences: NexusDataset::begin().finish("sequences"),
            counts: NexusGroup::new("counts"),
        }
    }

    fn create(&mut self, this: &hdf5::Group) {
        self.number.create(this);
        self.period_types.create(this);
        self.frames_requested.create(this);
        self.output.create(this);
        self.labels.create(this);
        self.raw_frames.create(this);
        self.good_frames.create(this);
        self.sequences.create(this);
        self.counts.create(this);
    }

    fn open(&mut self, this: &hdf5::Group) {
        self.number.open(this);
        self.period_types.open(this);
        self.frames_requested.open(this);
        self.output.open(this);
        self.labels.open(this);
        self.raw_frames.open(this);
        self.good_frames.open(this);
        self.sequences.open(this);
        self.counts.open(this);
    }

    fn close(&mut self) {
        self.number.close();
        self.period_types.close();
        self.frames_requested.close();
        self.output.close();
        self.labels.close();
        self.raw_frames.close();
        self.good_frames.close();
        self.sequences.close();
        self.counts.close();
    }
}

impl<'a> NxPushMessage<RunStart<'a>> for Periods {
    type MessageType = RunStart<'a>;
    
    fn push_message(&mut self, message: &Self::MessageType) {
        todo!()
    }
}