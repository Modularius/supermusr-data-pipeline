use hdf5::{H5Type, Location};
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

/// Implemented for objects who are constructed by a builder
/// i.e. NexusDataset and NexusAttribute instances
pub(super) trait NexusBuildable: Sized {
    type Builder: NexusBuilderBegun;

    fn begin(name: &str) -> Self::Builder;
}

/// Implemented for builders which require input
/// i.e. NexusBuilder with FINISHED = false
pub(super) trait NexusBuilderBegun {
    type FinshedBuilder: NexusBuilderFinished;
    
    fn new(name: &str) -> Self;
}

/// Implemented for builders which are ready to complete
/// i.e. NexusBuilder with FINISHED = true
pub(super) trait NexusBuilderFinished {
    type BuildType: NexusBuildable;

    fn finish(self) -> Self::BuildType;
}

/// Implemented for objects which can hold data
/// i.e. NexusBuilder with FINISHED = true
pub(super) trait NexusDataHolder: NexusBuildable {
    type DataType: H5Type + Default + Clone;

    type HDF5Type;
    type HDF5Container;

    fn create_hdf5(&self, parent: &Self::HDF5Container) -> Result<(), NexusError>;
    fn close_hdf5(&mut self);
}

/// Implemented for `NexusDataHolder` objects have mutable scalar data
/// i.e. NexusDataset and NexusAttribute instances with C = NexusDataHolderMutable
pub(super) trait NexusDataHolderScalarMutable: NexusDataHolder {
    fn write_scalar(&self, value: Self::DataType) -> Result<(), NexusError>;
    fn read_scalar(&self) -> Result<Self::DataType, NexusError>;
}

/// Implemented for `NexusDataHolder` objects have extendable vector data
/// i.e. NexusDataset and NexusAttribute instances with C = NexusDataHolderResizable
pub(super) trait NexusDataHolderAppendable: NexusDataHolder {
    fn append(&self, values: &[Self::DataType]) -> Result<(), NexusError>;
    fn get_size(&self) -> Result<usize, NexusError>;
}

/// Implemented for objects in `builder.rs` which serve as classes for `NexusDataHolder` objects
/// i.e. `NexusDataMutable`, `NexusDataHolderConstant` and `NexusDataHolderResizable`
pub(super) trait NexusDataHolderClass: Default + Clone {}

/// Implemented for structs in the `groups` folder which define the HDF5 group structure
pub(super) trait NexusGroupDef: Sized {
    const CLASS_NAME: &'static str;

    fn new() -> Self;
}

/// Implemented for structs in the `groups` folder which define the HDF5 dataset structure
pub(super) trait NexusDatasetDef: Sized {
    const UNITS: Option<NexusUnits> = None;

    fn new() -> Self;
}

impl NexusDatasetDef for () {
    fn new() -> Self {
        ()
    }
}

/// Implemented for structs in the `groups` folder which react immutably to `flatbuffer` messages
pub(crate) trait NexusPushMessage<P, M> {
    fn push_message(&self, message: &M, parent: &P) -> Result<(), NexusError>;
}

/// Implemented for structs in the `groups` folder which react mutably to `flatbuffer` messages
pub(crate) trait NexusPushMessageMut<P, M> {
    fn push_message_mut(&mut self, message: &M, parent: &P) -> Result<(), NexusError>;
}
