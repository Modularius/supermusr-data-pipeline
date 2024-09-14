use hdf5::Group;
use supermusr_streaming_types::{
    ecs_al00_alarm_generated::Alarm, ecs_f144_logdata_generated::f144_LogData,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};

use crate::{
    nexus::{nexus_class, NexusSettings},
    schematic::{
        elements::{
            attribute::NexusAttribute,
            dataset::{NexusDataset, NexusDatasetResize},
            NexusBuildable, NexusDataHolder, NexusDataHolderAppendable, NexusDatasetDef,
            NexusError, NexusGroupDef, NexusHandleMessage, NexusUnits,
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
            offset: NexusAttribute::begin("offset").finish_with_auto_default(),
        }
    }
}

pub(super) struct Log {
    time: NexusDatasetResize<i64, TimeAttributes>,
    value: NexusDatasetResize<u32>,
}

impl NexusGroupDef for Log {
    const CLASS_NAME: &'static str = nexus_class::LOG;
    type Settings = NexusSettings;

    fn new(settings: &Self::Settings) -> Self {
        Self {
            time: NexusDataset::begin("time").finish_with_resizable(
                Default::default(),
                0,
                settings.runloglist_chunk_size,
            ),
            value: NexusDataset::begin("value").finish_with_resizable(
                Default::default(),
                0,
                settings.runloglist_chunk_size,
            ),
        }
    }
}

impl<'a> NexusHandleMessage<f144_LogData<'a>> for Log {
    fn handle_message(
        &mut self,
        message: &f144_LogData<'a>,
        parent: &Group,
    ) -> Result<(), NexusError> {
        self.time.append(parent, &[message.timestamp()])?;
        self.value.append(
            parent,
            &[message.value_as_uint().ok_or(NexusError::Unknown)?.value()],
        )?;
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
    type Settings = NexusSettings;

    fn new(settings: &Self::Settings) -> Self {
        Self {
            alarm_severity: NexusDataset::begin("alarm_severity").finish_with_resizable(
                Default::default(),
                0,
                settings.seloglist_chunk_size,
            ),
            alarm_status: NexusDataset::begin("alarm_status").finish_with_resizable(
                Default::default(),
                0,
                settings.seloglist_chunk_size,
            ),
            alarm_time: NexusDataset::begin("alarm_time").finish_with_resizable(
                Default::default(),
                0,
                settings.seloglist_chunk_size,
            ),
            time: NexusDataset::begin("time").finish_with_resizable(
                Default::default(),
                0,
                settings.seloglist_chunk_size,
            ),
            value: NexusDataset::begin("value").finish_with_resizable(
                Default::default(),
                0,
                settings.seloglist_chunk_size,
            ),
        }
    }
}

impl<'a> NexusHandleMessage<se00_SampleEnvironmentData<'a>> for ValueLog {
    fn handle_message(
        &mut self,
        message: &se00_SampleEnvironmentData<'a>,
        parent: &Group,
    ) -> Result<(), NexusError> {
        self.time.append(
            parent,
            &message
                .timestamps()
                .ok_or(NexusError::Unknown)?
                .iter()
                .collect::<Vec<_>>(),
        )?;
        self.time.close_hdf5();

        self.value.append(
            parent,
            &message
                .values_as_uint_32_array()
                .ok_or(NexusError::Unknown)?
                .value()
                .iter()
                .collect::<Vec<_>>(),
        )?;
        Ok(())
    }
}

impl<'a> NexusHandleMessage<Alarm<'a>> for ValueLog {
    fn handle_message(&mut self, message: &Alarm<'a>, parent: &Group) -> Result<(), NexusError> {
        self.alarm_severity.append(
            parent,
            &[message
                .severity()
                .variant_name()
                .ok_or(NexusError::Unknown)?
                .parse()?],
        )?;
        self.alarm_status.append(
            parent,
            &[message
                .message()
                .ok_or(NexusError::Unknown)?
                .parse()
                .unwrap()],
        )?;
        self.alarm_time.append(parent, &[message.timestamp()])?;
        Ok(())
    }
}
