use builder::{NexusBuilder, NexusLogValueResizable};
use hdf5::{types::{StringError, TypeDescriptor}, Dataset, Group, H5Type};
use log_value::VectorOfScalars;
use thiserror::Error;

use crate::error::{NexusDatasetError, NexusPushError};

pub(crate) mod attribute;
pub(crate) mod builder;
pub(crate) mod dataset;
pub(crate) mod group;
pub(crate) mod log_value;

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
pub(super) trait NexusBuilderBegun: Sized {
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
    //type DataType: H5Type + Default + Clone;

    type HDF5Type;
    type HDF5Container;
    type ThisError;

    fn create_hdf5(&mut self, parent: &Self::HDF5Container) -> Result<(), Self::ThisError>;
    fn create_hdf5_instance(
        &self,
        parent: &Self::HDF5Container,
    ) -> Result<Self::HDF5Type, Self::ThisError>;
    fn close_hdf5(&mut self);
}

/// Implemented for objects which can hold data
/// i.e. NexusBuilder with FINISHED = true
pub(super) trait NexusTypedDataHolder: NexusDataHolder {
    type DataType: H5Type + Default + Clone;
}

/// Implemented for `NexusDataHolder` objects have mutable scalar data
/// i.e. NexusDataset and NexusAttribute instances with C = NexusDataHolderMutable
pub(super) trait NexusDataHolderScalarMutable: NexusTypedDataHolder {
    fn write_scalar(
        &self,
        parent: &Self::HDF5Container,
        value: Self::DataType,
    ) -> Result<(), Self::ThisError>;
    fn read_scalar(&self, parent: &Self::HDF5Container) -> Result<Self::DataType, Self::ThisError>;
}

/// Implemented for `NexusDataHolder` objects have extendable vector data
/// i.e. NexusDataset and NexusAttribute instances with C = NexusDataHolderResizable
pub(super) trait NexusDataHolderSizable: NexusDataHolder {
    fn get_size(&self, parent: &Self::HDF5Container) -> Result<usize, Self::ThisError>;
}

/// Implemented for `NexusDataHolder` objects have extendable vector data
/// i.e. NexusDataset and NexusAttribute instances with C = NexusDataHolderResizable
pub(super) trait NexusDataHolderAppendable:
    NexusTypedDataHolder + NexusDataHolderSizable
{
    fn append(
        &self,
        parent: &Self::HDF5Container,
        values: &[Self::DataType],
    ) -> Result<(), Self::ThisError>;
}

/// Implemented for `NexusDataHolder` objects have extendable vector data
/// i.e. NexusDataset and NexusAttribute instances with C = NexusDataHolderResizable
pub(super) trait NexusLogValueDataHolderAppendable: NexusDataHolderSizable {
    fn append(
        &self,
        parent: &Self::HDF5Container,
        values: &VectorOfScalars,
    ) -> Result<(), Self::ThisError>;
}

/// Implemented for objects in `builder.rs` which serve as classes for `NexusDataHolder` objects
/// i.e. `NexusDataMutable`, `NexusDataHolderConstant` and `NexusDataHolderResizable`
pub(super) trait NexusClassDataHolder: Default + Clone {}

/// Implemented for structs in the `groups` folder which define the HDF5 group structure
pub(crate) trait NexusGroupDef: Sized {
    const CLASS_NAME: &'static str;
    type Settings;

    fn new(_settings: &Self::Settings) -> Self;
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

/// Implemented for structs in the `groups` folder which define the HDF5 dataset structure
pub(super) trait NexusDatasetDefWithTypeDesciptor: NexusDatasetDef {
    fn type_descriptor() -> Option<TypeDescriptor>;
}

/// Implemented for NexusGroup and NexusDataset instances which react immutably to `flatbuffer` messages M
/// R is an optional return value
pub(crate) trait NexusPushMessage<M, P = Group, R = ()> {
    fn push_message(&mut self, message: &M, parent: &P) -> Result<R, NexusPushError>;
}

/// Implemented for structs in the `groups` folder which react immutably to `flatbuffer` messages M
/// R is an optional return value
pub(crate) trait NexusHandleMessage<M, P = Group, R = ()> {
    fn handle_message(&mut self, message: &M, own: &P) -> Result<R, NexusPushError>;
}

/// Same as NexusPushMessage but allows additional mutable context to be added
pub(crate) trait NexusPushMessageWithContext<M, P = Group, R = ()> {
    type Context;

    fn push_message_with_context(
        &mut self,
        message: &M,
        parent: &P,
        context: &mut Self::Context,
    ) -> Result<R, NexusPushError>;
}

/// Same as NexusHandleMessage but allows additional mutable context to be added
pub(crate) trait NexusHandleMessageWithContext<M, P = Group, R = ()> {
    type Context;

    fn handle_message_with_context(
        &mut self,
        message: &M,
        own: &P,
        context: &mut Self::Context,
    ) -> Result<R, NexusPushError>;
}
