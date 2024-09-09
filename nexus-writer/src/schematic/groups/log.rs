use supermusr_streaming_types::{
    ecs_al00_alarm_generated::Alarm, ecs_f144_logdata_generated::f144_LogData,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};

use crate::{
    nexus::nexus_class,
    schematic::{
        elements::{
            attribute::{NexusAttribute, NexusUnits},
            dataset::{AttributeRegister, NexusDataset, NexusDatasetResize, NxDataset},
            group::{GroupContentRegister, NxGroup, NxPushMessage},
            traits::{Buildable, CanAppend},
        },
        H5String,
    },
};

#[derive(Clone)]
struct TimeAttributes {
    offset: NexusAttribute<H5String>,
}

impl NxDataset for TimeAttributes {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Nanoseconds);

    fn new(attribute_register: AttributeRegister) -> Self {
        Self {
            offset: NexusAttribute::begin("offset")
                .default_value(Default::default())
                .finish(&attribute_register),
        }
    }
}

pub(super) struct Log {
    time: NexusDatasetResize<i64, TimeAttributes>,
    value: NexusDatasetResize<u32>,
}

impl NxGroup for Log {
    const CLASS_NAME: &'static str = nexus_class::LOG;

    fn new(dataset_register: GroupContentRegister) -> Self {
        Self {
            time: NexusDataset::begin("time")
                .resizable(Default::default(), 0, 128)
                .finish(&dataset_register),
            value: NexusDataset::begin("value")
                .resizable(Default::default(), 0, 128)
                .finish(&dataset_register),
        }
    }
}

impl<'a> NxPushMessage<f144_LogData<'a>> for Log {
    type MessageType = f144_LogData<'a>;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.time.append(&[message.timestamp()])?;
        self.value
            .append(&[message.value_as_uint().unwrap().value()])?;
        Ok(())
    }
}

pub(super) struct ValueLog {
    alarm_severity: NexusDatasetResize<H5String>,
    alarm_status: NexusDatasetResize<H5String>,
    alarm_time: NexusDatasetResize<i64>,
    time: NexusDatasetResize<i64, TimeAttributes>,
    value: NexusDatasetResize<u32>,
}

impl NxGroup for ValueLog {
    const CLASS_NAME: &'static str = nexus_class::LOG;

    fn new(dataset_register: GroupContentRegister) -> Self {
        Self {
            alarm_severity: NexusDataset::begin("alarm_severity")
                .resizable(Default::default(), 0, 128)
                .finish(&dataset_register),
            alarm_status: NexusDataset::begin("alarm_status")
                .resizable(Default::default(), 0, 128)
                .finish(&dataset_register),
            alarm_time: NexusDataset::begin("alarm_time")
                .resizable(Default::default(), 0, 128)
                .finish(&dataset_register),
            time: NexusDataset::begin("time")
                .resizable(Default::default(), 0, 128)
                .finish(&dataset_register),
            value: NexusDataset::begin("value")
                .resizable(Default::default(), 0, 128)
                .finish(&dataset_register),
        }
    }
}

impl<'a> NxPushMessage<se00_SampleEnvironmentData<'a>> for ValueLog {
    type MessageType = se00_SampleEnvironmentData<'a>;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.time
            .append(&message.timestamps().unwrap().iter().collect::<Vec<_>>())?;
        self.value.append(
            &message
                .values_as_uint_32_array()
                .unwrap()
                .value()
                .iter()
                .collect::<Vec<_>>(),
        )?;
        Ok(())
    }
}

impl<'a> NxPushMessage<Alarm<'a>> for ValueLog {
    type MessageType = Alarm<'a>;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.alarm_severity.append(&[message
            .severity()
            .variant_name()
            .unwrap()
            .parse()
            .unwrap()])?;
        self.alarm_status
            .append(&[message.message().unwrap().parse().unwrap()])?;
        self.alarm_time.append(&[message.timestamp()])?;
        Ok(())
    }
}
