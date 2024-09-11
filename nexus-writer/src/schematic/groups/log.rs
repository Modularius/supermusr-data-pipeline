use hdf5::{Group, Location};
use supermusr_streaming_types::{
    ecs_al00_alarm_generated::Alarm, ecs_f144_logdata_generated::f144_LogData,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};

use crate::{
    nexus::nexus_class,
    schematic::{
        elements::{
            attribute::NexusAttribute, dataset::{NexusDataset, NexusDatasetResize}, NexusBuildable, NexusBuilderFinished, NexusDataHolder, NexusDataHolderAppendable, NexusDatasetDef, NexusError, NexusGroupDef, NexusHandleMessage, NexusPushMessage, NexusUnits
        },
        H5String,
    },
};

#[derive(Clone)]
struct TimeAttributes {
    offset: NexusAttribute<H5String>,
}

impl NexusDatasetDef for TimeAttributes {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Nanoseconds);

    fn new() -> Self {
        Self {
            offset: NexusAttribute::begin("offset")
                .default_value(Default::default())
                .finish(),
        }
    }
}

pub(super) struct Log {
    time: NexusDatasetResize<i64, TimeAttributes>,
    value: NexusDatasetResize<u32>,
}

impl NexusGroupDef for Log {
    const CLASS_NAME: &'static str = nexus_class::LOG;

    fn new() -> Self {
        Self {
            time: NexusDataset::begin("time")
                .resizable(Default::default(), 0, 128)
                .finish(),
            value: NexusDataset::begin("value")
                .resizable(Default::default(), 0, 128)
                .finish(),
        }
    }
}

impl<'a> NexusHandleMessage<f144_LogData<'a>> for Log {
    fn handle_message(&mut self, message: &f144_LogData<'a>, parent: &Group) -> Result<(), NexusError> {
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

impl NexusGroupDef for ValueLog {
    const CLASS_NAME: &'static str = nexus_class::LOG;

    fn new() -> Self {
        Self {
            alarm_severity: NexusDataset::begin("alarm_severity")
                .resizable(Default::default(), 0, 128)
                .finish(),
            alarm_status: NexusDataset::begin("alarm_status")
                .resizable(Default::default(), 0, 128)
                .finish(),
            alarm_time: NexusDataset::begin("alarm_time")
                .resizable(Default::default(), 0, 128)
                .finish(),
            time: NexusDataset::begin("time")
                .resizable(Default::default(), 0, 128)
                .finish(),
            value: NexusDataset::begin("value")
                .resizable(Default::default(), 0, 128)
                .finish(),
        }
    }
}

impl<'a> NexusHandleMessage<se00_SampleEnvironmentData<'a>> for ValueLog {
    fn handle_message(&mut self, message: &se00_SampleEnvironmentData<'a>, parent: &Group) -> Result<(), NexusError> {
        self.time.create_hdf5(parent)?;
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
        self.time.close_hdf5();
        Ok(())
    }
}

impl<'a> NexusHandleMessage<Alarm<'a>> for ValueLog {
    fn handle_message(&mut self, message: &Alarm<'a>, parent: &Group) -> Result<(), NexusError> {
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
