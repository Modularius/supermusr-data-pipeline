use hdf5::types::VarLenAscii;
use supermusr_streaming_types::{
    ecs_al00_alarm_generated::Alarm, ecs_f144_logdata_generated::f144_LogData,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};

use crate::{
    nexus::nexus_class,
    schematic::elements::{
        attribute::{NexusAttribute, NexusUnits, RcNexusAttributeVar},
        dataset::{
            CanAppend, NexusDataset, NxContainerAttributes, RcAttributeRegister,
            RcNexusDatasetResize,
        },
        group::{NxGroup, NxPushMessage, RcGroupContentRegister},
    },
};

#[derive(Clone)]
struct TimeAttributes {
    offset: RcNexusAttributeVar<VarLenAscii>,
}

impl NxContainerAttributes for TimeAttributes {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Nanoseconds);

    fn new(attribute_register: RcAttributeRegister) -> Self {
        Self {
            offset: NexusAttribute::begin().finish("offset", attribute_register.clone()),
        }
    }
}

pub(super) struct Log {
    time: RcNexusDatasetResize<i64, TimeAttributes>,
    value: RcNexusDatasetResize<u32>,
}

impl NxGroup for Log {
    const CLASS_NAME: &'static str = nexus_class::LOG;

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            time: NexusDataset::begin()
                .resizable(0, 128)
                .finish("time", dataset_register.clone()),
            value: NexusDataset::begin()
                .resizable(0, 128)
                .finish("value", dataset_register.clone()),
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
    alarm_severity: RcNexusDatasetResize<VarLenAscii>,
    alarm_status: RcNexusDatasetResize<VarLenAscii>,
    alarm_time: RcNexusDatasetResize<i64>,
    time: RcNexusDatasetResize<i64, TimeAttributes>,
    value: RcNexusDatasetResize<u32>,
}

impl NxGroup for ValueLog {
    const CLASS_NAME: &'static str = nexus_class::LOG;

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            alarm_severity: NexusDataset::begin()
                .resizable(0, 128)
                .finish("alarm_severity", dataset_register.clone()),
            alarm_status: NexusDataset::begin()
                .resizable(0, 128)
                .finish("alarm_status", dataset_register.clone()),
            alarm_time: NexusDataset::begin()
                .resizable(0, 128)
                .finish("alarm_time", dataset_register.clone()),
            time: NexusDataset::begin()
                .resizable(0, 128)
                .finish("time", dataset_register.clone()),
            value: NexusDataset::begin()
                .resizable(0, 128)
                .finish("value", dataset_register.clone()),
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
        self.alarm_severity
            .append(&[
                VarLenAscii::from_ascii(message.severity().variant_name().unwrap()).unwrap(),
            ])?;
        self.alarm_status
            .append(&[VarLenAscii::from_ascii(message.message().unwrap()).unwrap()])?;
        self.alarm_time.append(&[message.timestamp()])?;
        Ok(())
    }
}
