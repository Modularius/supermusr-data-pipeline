use hdf5::H5Type;
use thiserror::Error;

pub(crate) mod attribute;
pub(crate) mod builder;
pub(crate) mod dataset;
pub(crate) mod group;

#[derive(Debug, Error)]
pub(crate) enum NexusError {
    #[error("Error")]
    Unknown,
}

pub(crate) trait NexusBuildable: Sized {
    type Builder: NexusBuilderBegun;

    fn begin(name: &str) -> Self::Builder;
}

pub(crate) trait NexusBuilderBegun {
    type FinshedBuilder: NexusBuilderFinished;
    
    fn new(name: &str) -> Self;
}

pub(crate) trait NexusBuilderFinished {
    type BuiltType: NexusBuildable;

    fn finish(self) -> Self::BuiltType;
}

pub(crate) trait NexusDataHolder: NexusBuildable {
    type DataType: H5Type + Default + Clone;

    type HDF5Type;
    type HDF5Container;

    fn create_hdf5(&mut self, parent: &Self::HDF5Container) -> Result<(), NexusError>;
    fn close_hdf5(&mut self);
}

pub(crate) trait NexusDataHolderScalarMutable: NexusDataHolder {
    fn write_scalar(&self, value: Self::DataType) -> Result<(), NexusError>;
    fn read_scalar(&self) -> Result<Self::DataType, NexusError>;
}

pub(crate) trait NexusDataHolderAppendable: NexusDataHolder {
    fn append(&self, values: &[Self::DataType]) -> Result<(), NexusError>;
    fn get_size(&self) -> Result<usize, NexusError>;
}

pub(crate) trait NexusDataHolderClass: Default + Clone {}

pub(crate) trait NexusGroupDef: Sized {
    const CLASS_NAME: &'static str;

    fn new() -> Self;
}

#[derive(strum::Display)]
pub(crate) enum NexusUnits {
    #[strum(to_string = "second")]
    Seconds,
    #[strum(to_string = "us")]
    Microseconds,
    #[strum(to_string = "ns")]
    Nanoseconds,
    #[strum(to_string = "ISO8601")]
    ISO8601,
    #[strum(to_string = "mEv")]
    MegaElectronVolts,
    #[strum(to_string = "uAh")]
    MicroAmpHours,
    #[strum(to_string = "counts")]
    Counts,
}

pub(crate) trait NexusDatasetDef: Sized {
    const UNITS: Option<NexusUnits> = None;

    fn new() -> Self;
}

impl NexusDatasetDef for () {
    fn new() -> Self {
        ()
    }
}

pub(crate) trait NexusPushMessage<T> {
    type MessageType;

    fn push_message(&self, message: &Self::MessageType) -> Result<(), NexusError>;
}

pub(crate) trait NexusPushMessageMut<T> {
    type MessageType;

    fn push_message_mut(&mut self, message: &Self::MessageType) -> Result<(), NexusError>;
}
