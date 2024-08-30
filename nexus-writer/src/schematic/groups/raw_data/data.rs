use supermusr_streaming_types::aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage;

use crate::schematic::{
    elements::{
        attribute::{NexusAttribute, NexusUnits},
        dataset::{AttributeRegister, NexusDataset, NexusDatasetResize, NxDataset},
        group::{GroupContentRegister, NxGroup, NxPushMessage},
        traits::{Buildable, CanAppend},
    },
    nexus_class, H5String,
};

#[derive(Clone)]
struct EventTimeOffsetAttributes {}

impl NxDataset for EventTimeOffsetAttributes {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Nanoseconds);

    fn new(_attribute_register: AttributeRegister) -> Self {
        Self {}
    }
}

#[derive(Clone)]
struct EventTimeZeroAttributes {
    offset: NexusAttribute<H5String>,
}

impl NxDataset for EventTimeZeroAttributes {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Nanoseconds);

    fn new(attribute_register: AttributeRegister) -> Self {
        Self {
            offset: NexusAttribute::begin("offset").finish(&attribute_register),
        }
    }
}

pub(super) struct Data {
    event_id: NexusDatasetResize<u32>,
    event_index: NexusDatasetResize<u32>,
    event_time_offset: NexusDatasetResize<u32, EventTimeOffsetAttributes>,
    event_time_zero: NexusDatasetResize<u64, EventTimeZeroAttributes>,
    event_period_number: NexusDatasetResize<u64>,
    event_pulse_height: NexusDatasetResize<f64>,
}

impl NxGroup for Data {
    const CLASS_NAME: &'static str = nexus_class::EVENT_DATA;

    fn new(dataset_register: GroupContentRegister) -> Self {
        Self {
            event_id: NexusDataset::begin("event_id")
                .resizable(0, 128)
                .finish(&dataset_register),
            event_index: NexusDataset::begin("event_index")
                .resizable(0, 128)
                .finish(&dataset_register),
            event_time_offset: NexusDataset::begin("event_time_offset")
                .resizable(0, 128)
                .finish(&dataset_register),
            event_time_zero: NexusDataset::begin("event_time_zero")
                .resizable(0, 128)
                .finish(&dataset_register),
            event_period_number: NexusDataset::begin("event_period_number")
                .resizable(0, 128)
                .finish(&dataset_register),
            event_pulse_height: NexusDataset::begin("event_pulse_height")
                .resizable(0, 128)
                .finish(&dataset_register),
        }
    }
}

impl<'a> NxPushMessage<FrameAssembledEventListMessage<'a>> for Data {
    type MessageType = FrameAssembledEventListMessage<'a>;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        // Here is where we extend the datasets
        self.event_id
            .append(&message.channel().expect("").iter().collect::<Vec<_>>())?;
        //TODO
        Ok(())
    }
}
