use std::any::Any;

use hdf5::{
    types::{OwnedDynValue, TypeDescriptor},
    Datatype, Group, H5Type,
};
use supermusr_streaming_types::{
    ecs_al00_alarm_generated::Alarm,
    ecs_f144_logdata_generated::{f144_LogData, Value},
    ecs_se00_data_generated::{se00_SampleEnvironmentData, ValueUnion},
};

use crate::{
    error::{
        NexusDatasetError, NexusLogValueError, NexusMissingAlarmError, NexusMissingError,
        NexusMissingRunlogError, NexusMissingSelogError, NexusPushError,
    },
    nexus::{nexus_class, NexusSettings},
    schematic::{
        elements::{
            attribute::NexusAttribute,
            dataset::{
                NexusDataset, NexusDatasetMut, NexusDatasetResize, NexusLogValueDatasetResize,
            },
            log_value::NumericVector,
            NexusBuildable, NexusBuilderBegun, NexusDataHolder, NexusDataHolderAppendable,
            NexusDatasetDef, NexusGroupDef, NexusHandleMessage, NexusLogValueDataHolderAppendable,
            NexusUnits,
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
    type_desc: Option<TypeDescriptor>,
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
            value: NexusDataset::begin("value")
                .finish_log_value_with_resizable(settings.runloglist_chunk_size),
            type_desc: None,
        }
    }
}

fn get_value<T>(val: Option<T>) -> Result<T, NexusMissingError> {
    val.ok_or(NexusMissingRunlogError::Message)
        .map_err(NexusMissingError::Runlog)
}

impl<'a> TryFrom<&f144_LogData<'a>> for VectorOfScalars {
    type Error = NexusPushError;

    fn try_from(value: &f144_LogData<'a>) -> Result<Self, NexusPushError> {
        Ok(match value.value_type() {
            Value::Byte => Self::I1(vec![get_value(value.value_as_byte())?.value()]),
            Value::Short => Self::I2(vec![get_value(value.value_as_short())?.value()]),
            Value::Int => Self::I4(vec![get_value(value.value_as_int())?.value()]),
            Value::Long => Self::I8(vec![get_value(value.value_as_long())?.value()]),
            Value::UByte => Self::U1(vec![get_value(value.value_as_ubyte())?.value()]),
            Value::UShort => Self::U2(vec![get_value(value.value_as_ushort())?.value()]),
            Value::UInt => Self::U4(vec![get_value(value.value_as_uint())?.value()]),
            Value::ULong => Self::U8(vec![get_value(value.value_as_ulong())?.value()]),
            Value::Float => Self::F4(vec![get_value(value.value_as_float())?.value()]),
            Value::Double => Self::F8(vec![get_value(value.value_as_double())?.value()]),
            value => Err(NexusLogValueError::InvalidRunLogType { value })?,
        })
    }
}

impl<'a> NexusHandleMessage<f144_LogData<'a>> for Log {
    fn handle_message(
        &mut self,
        message: &f144_LogData<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        self.time.append(parent, &[message.timestamp()])?;

        let value: VectorOfScalars = message.try_into()?;
        if let Some(type_desc) = self.type_desc {
            if type_desc != value.type_descriptor() {
                return Err(NexusLogValueError::TypeMismatch {
                    required_type: type_desc,
                    input_type: value.type_descriptor(),
                })?;
            }
        }
        self.value.append(parent, &value)?;
        Ok(())
    }
}

pub(super) struct ValueLog {
    alarm_severity: NexusDatasetResize<H5String>,
    alarm_status: NexusDatasetResize<H5String>,
    alarm_time: NexusDatasetResize<i64>,
    time: NexusDatasetResize<i64, TimeAttributes>,
    value: NexusLogValueDatasetResize,
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
            value: NexusDataset::begin("value")
                .finish_log_value_with_resizable(settings.seloglist_chunk_size),
        }
    }
}

impl<'a> TryFrom<&se00_SampleEnvironmentData<'a>> for VectorOfScalars {
    type Error = NexusPushError;

    fn try_from(value: &se00_SampleEnvironmentData<'a>) -> Result<Self, NexusPushError> {
        Ok(match value.values_type() {
            ValueUnion::Int8Array => Self::I1(
                get_value(value.values_as_int_8_array())?
                    .value()
                    .iter()
                    .collect(),
            ),
            ValueUnion::Int16Array => Self::I2(
                get_value(value.values_as_int_16_array())?
                    .value()
                    .iter()
                    .collect(),
            ),
            ValueUnion::Int32Array => Self::I4(
                get_value(value.values_as_int_32_array())?
                    .value()
                    .iter()
                    .collect(),
            ),
            ValueUnion::Int64Array => Self::I8(
                get_value(value.values_as_int_64_array())?
                    .value()
                    .iter()
                    .collect(),
            ),
            ValueUnion::UInt8Array => Self::U1(
                get_value(value.values_as_uint_8_array())?
                    .value()
                    .iter()
                    .collect(),
            ),
            ValueUnion::UInt16Array => Self::U2(
                get_value(value.values_as_uint_16_array())?
                    .value()
                    .iter()
                    .collect(),
            ),
            ValueUnion::UInt32Array => Self::U4(
                get_value(value.values_as_uint_32_array())?
                    .value()
                    .iter()
                    .collect(),
            ),
            ValueUnion::UInt64Array => Self::U8(
                get_value(value.values_as_uint_64_array())?
                    .value()
                    .iter()
                    .collect(),
            ),
            ValueUnion::FloatArray => Self::F4(
                get_value(value.values_as_float_array())?
                    .value()
                    .iter()
                    .collect(),
            ),
            ValueUnion::DoubleArray => Self::F8(
                get_value(value.values_as_double_array())?
                    .value()
                    .iter()
                    .collect(),
            ),
            value => Err(NexusPushError::InvalidSelogType { value })?,
        })
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

        self.value.append(parent, &message.try_into()?)?;
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
