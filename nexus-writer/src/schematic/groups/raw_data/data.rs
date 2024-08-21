use hdf5::{
    types::{IntSize, TypeDescriptor, VarLenAscii},
    Group,
};
use supermusr_streaming_types::aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage;

use crate::schematic::elements::{
    attribute::{NexusAttribute, NexusUnits},
    dataset::{MustEnterAttributes, NexusDataset},
    group::{NxGroup, NxPushMessage},
};

pub(super) struct Data {
    event_id: NexusDataset<u32>,
    event_index: NexusDataset<u32>,
    event_time_offset: NexusDataset<u32, MustEnterAttributes<1>>,
    event_time_zero: NexusDataset<u64, MustEnterAttributes<2>>,
    event_period_number: NexusDataset<u64>,
    event_pulse_height: NexusDataset<f64>,
}

impl NxGroup for Data {
    const CLASS_NAME: &'static str = "NXperiod";

    fn new() -> Self {
        Self {
            event_id: NexusDataset::begin().finish("event_id"),
            event_index: NexusDataset::begin().finish("event_index"),
            event_time_offset: NexusDataset::begin()
                .attributes([NexusAttribute::units(NexusUnits::Nanoseconds)])
                .finish("event_time_offset"),
            event_time_zero: NexusDataset::begin()
                .attributes([
                    NexusAttribute::units(NexusUnits::Nanoseconds),
                    NexusAttribute::new("Start", TypeDescriptor::Integer(IntSize::U4)),
                ])
                .finish("event_time_zero"),
            event_period_number: NexusDataset::begin().finish("event_period_number"),
            event_pulse_height: NexusDataset::begin().finish("event_pulse_height"),
        }
    }

    fn create(&mut self, this: &Group) {
        self.event_id.create(this);
        self.event_index.create(this);
        self.event_time_offset.create(this);
        self.event_time_zero.create(this);
        self.event_period_number.create(this);
        self.event_pulse_height.create(this);
    }

    fn open(&mut self, this: &Group) {
        self.event_id.open(this);
        self.event_index.open(this);
        self.event_time_offset.open(this);
        self.event_time_zero.open(this);
        self.event_period_number.open(this);
        self.event_pulse_height.open(this);
    }

    fn close(&mut self) {
        self.event_id.close();
        self.event_index.close();
        self.event_time_offset.close();
        self.event_time_zero.close();
        self.event_period_number.close();
        self.event_pulse_height.close();
    }
}


impl<'a> NxPushMessage<FrameAssembledEventListMessage<'a>> for Data {
    type MessageType = FrameAssembledEventListMessage<'a>;

    fn push_message(&mut self, message: &Self::MessageType) {
        // Here is where we extend the datasets
    }
}