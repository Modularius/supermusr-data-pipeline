use hdf5::{types::TypeDescriptor, Group};
use supermusr_streaming_types::{
    ecs_al00_alarm_generated::Alarm, ecs_f144_logdata_generated::f144_LogData,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};

use crate::{
    error::{
        NexusMissingAlarmError, NexusMissingError, NexusMissingRunlogError, NexusMissingSelogError,
        NexusPushError,
    },
    nexus::{nexus_class, NexusSettings},
    schematic::{
        elements::{
            attribute::NexusAttribute,
            dataset::{NexusDataset, NexusDatasetResize, NexusLogValueDatasetResize},
            NexusBuildable, NexusDataHolder, NexusDataHolderAppendable, NexusDatasetDef,
            NexusGroupDef, NexusHandleMessage, NexusUnits,
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
    value: NexusLogValueDatasetResize,
}

impl NexusGroupDef for Log {
    const CLASS_NAME: &'static str = nexus_class::LOG;
    type Settings = (NexusSettings,TypeDescriptor);

    fn new((settings, type_desc): &Self::Settings) -> Self {
        Self {
            time: NexusDataset::begin("time").finish_with_resizable(
                Default::default(),
                0,
                settings.runloglist_chunk_size,
            ),
            value: NexusDataset::begin("value")
            .finish_log_value_with_resizable(
                settings.runloglist_chunk_size,
                TypeDescriptor::Boolean,
            ),
        }
    }
}

impl<'a> NexusHandleMessage<f144_LogData<'a>> for Log {
    fn handle_message(
        &mut self,
        message: &f144_LogData<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        self.time.append(parent, &[message.timestamp()])?;

        
        let mut log = match message.value_type() {
            supermusr_streaming_types::ecs_f144_logdata_generated::Value::Byte => message.value_as_byte(),
            supermusr_streaming_types::ecs_f144_logdata_generated::Value::Short => message.value_as_short(),
            supermusr_streaming_types::ecs_f144_logdata_generated::Value::Int => message.value_as_int(),
            supermusr_streaming_types::ecs_f144_logdata_generated::Value::Long => message.value_as_long(),
            supermusr_streaming_types::ecs_f144_logdata_generated::Value::UByte => message.value_as_ubyte(),
            supermusr_streaming_types::ecs_f144_logdata_generated::Value::UShort => message.value_as_ushort(),
            supermusr_streaming_types::ecs_f144_logdata_generated::Value::UInt => message.value_as_uint(),
            supermusr_streaming_types::ecs_f144_logdata_generated::Value::ULong => message.value_as_ulong(),
            supermusr_streaming_types::ecs_f144_logdata_generated::Value::Float => message.value_as_float(),
            supermusr_streaming_types::ecs_f144_logdata_generated::Value::Float => message.value_as_double()
        };
        self.value.append(
            parent,
            &[message
                .value_as_uint()
                .ok_or(NexusMissingRunlogError::Message)
                .map_err(NexusMissingError::Runlog)?
                .value()],
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
    ) -> Result<(), NexusPushError> {
        self.time.append(
            parent,
            &message
                .timestamps()
                .ok_or(NexusMissingSelogError::Times)
                .map_err(NexusMissingError::Selog)?
                .iter()
                .collect::<Vec<_>>(),
        )?;
        self.time.close_hdf5();

        self.value.append(
            parent,
            &message
                .values_as_uint_32_array()
                .ok_or(NexusMissingSelogError::Message)
                .map_err(NexusMissingError::Selog)?
                .value()
                .iter()
                .collect::<Vec<_>>(),
        )?;
        Ok(())
    }
}

impl<'a> NexusHandleMessage<Alarm<'a>> for ValueLog {
    fn handle_message(
        &mut self,
        message: &Alarm<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        self.alarm_severity.append(
            parent,
            &[message
                .severity()
                .variant_name()
                .ok_or(NexusMissingAlarmError::Severity)
                .map_err(NexusMissingError::Alarm)?
                .parse()?],
        )?;
        self.alarm_status.append(
            parent,
            &[message
                .message()
                .ok_or(NexusMissingAlarmError::Message)
                .map_err(NexusMissingError::Alarm)?
                .parse()
                .unwrap()],
        )?;
        self.alarm_time.append(parent, &[message.timestamp()])?;
        Ok(())
    }
}
