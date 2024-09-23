use chrono::TimeDelta;
use hdf5::types::TypeDescriptor;
use supermusr_streaming_types::{
    ecs_f144_logdata_generated::Value, ecs_se00_data_generated::ValueUnion,
    time_conversions::GpsTimeConversionError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum HDF5Error {
    #[error("HDF5 Error {0}")]
    HDF5(#[from] hdf5::Error),
    #[error("HDF5 String Error: {0}")]
    HDF5String(#[from] hdf5::types::StringError),
}

#[derive(Debug, Error)]
pub(crate) enum NexusMissingAlarmError {
    #[error("Alarm Message Missing")]
    Message,
    #[error("Alarm Severity Missing")]
    Severity,
}

#[derive(Debug, Error)]
pub(crate) enum NexusMissingSelogError {
    #[error("Selog Message Missing")]
    Message,
    #[error("Selog Times Missing")]
    Times,
}

#[derive(Debug, Error)]
pub(crate) enum NexusMissingRunlogError {
    #[error("Runlog Message Missing")]
    Message,
}



#[derive(Debug, Error)]
pub(crate) enum NexusMissingRunStartError {
    #[error("Runstart Run Name Missing")]
    RunName,
}

#[derive(Debug, Error)]
pub(crate) enum NexusMissingEventlistError {
    #[error("Timestamp")]
    Timestamp,
    #[error("Channels")]
    Channels,
    #[error("Voltages")]
    Voltages,
    #[error("Times")]
    Times,
}

#[derive(Debug, Error)]
pub(crate) enum NexusMissingError {
    #[error("Alarm: {0}")]
    Alarm(NexusMissingAlarmError),
    #[error("Selog: {0}")]
    Selog(NexusMissingSelogError),
    #[error("Runlog: {0}")]
    Runlog(NexusMissingRunlogError),
    #[error("Eventlist {0}")]
    Eventlist(NexusMissingEventlistError),
    #[error("RunStart {0}")]
    RunStart(NexusMissingRunStartError)
}

#[derive(Debug, Error)]
pub(crate) enum NexusPushError {
    #[error("Dataset Error: {0}")]
    Group(#[from] NexusGroupError),
    #[error("Group Error: {0}")]
    Dataset(#[from] NexusDatasetError),
    #[error("Attribute Error: {0}")]
    Attribute(#[from] NexusAttributeError),
    #[error("Numeric Vector Error: {0}")]
    LogValue(#[from] NexusNumericError),
    #[error("HDF5 Error {0}")]
    HDF5(#[from] hdf5::Error),
    #[error("HDF5 String Error: {0}")]
    HDF5String(#[from] hdf5::types::StringError),
    #[error("HDF5 String Error: {0}")]
    MissingValue(#[from] NexusMissingError),
    #[error("Cannot fit duration into i64: {0}")]
    NanosecondError(TimeDelta),
    #[error("TimeDelta negative: {0}")]
    TimeDeltaNegative(<u64 as TryFrom<i64>>::Error),
    #[error("Chrono Parse Error: {0}")]
    ChronoParse(#[from] chrono::ParseError),
    #[error("Parse Error: {0}")]
    GpsTimeConversion(#[from] GpsTimeConversionError),
}

#[derive(Debug, Error)]
pub(crate) enum NexusDatasetError {
    #[error("HDF5 Error {0}")]
    HDF5(#[from] hdf5::Error),
    #[error("HDF5 String Error: {0}")]
    HDF5String(#[from] hdf5::types::StringError),
    #[error("Numeric Error: {0}")]
    Numeric(#[from] NexusNumericError),
}

#[derive(Debug, Error)]
pub(crate) enum NexusNumericError {
    #[error("HDF5 Error {0}")]
    HDF5(#[from] hdf5::Error),
    #[error("HDF5 String Error: {0}")]
    HDF5String(#[from] hdf5::types::StringError),
    #[error("Invalid Run Log Type of Value: {0:?}", value.variant_name())]
    InvalidRunLogType { value: Value },
    #[error("Invalid Selog Type of Value: {0:?}", value.variant_name())]
    InvalidSelogType { value: ValueUnion },
    #[error("Type Mismatch required: {0}, input: {1} ", required_type, input_type)]
    TypeMismatch {
        required_type: TypeDescriptor,
        input_type: TypeDescriptor,
    },
    #[error("Type Not Set")]
    NumericTypeNotSet,
}

#[derive(Debug, Error)]
pub(crate) enum NexusGroupError {
    #[error("HDF5 Error {0}")]
    HDF5(#[from] hdf5::Error),
    #[error("HDF5 String Error: {0}")]
    HDF5String(#[from] hdf5::types::StringError),
    #[error("Dataset Error: {source}")]
    Dataset {
        source: NexusDatasetError,
        path: String,
    },
}

#[derive(Debug, Error)]
pub(crate) enum NexusAttributeError {
    #[error("HDF5 Error {0}")]
    HDF5(#[from] hdf5::Error),
    #[error("HDF5 String Error: {0}")]
    HDF5String(#[from] hdf5::types::StringError),
}

#[derive(Debug, Error)]
pub(crate) enum NexusError {
    #[error("Error")]
    Unknown,
    #[error("Dataset Error: {0}")]
    Group(#[from] NexusGroupError),
    #[error("Group Error: {0}")]
    Dataset(#[from] NexusDatasetError),
}
