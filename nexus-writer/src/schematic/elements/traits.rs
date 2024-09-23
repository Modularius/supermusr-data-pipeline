use super::log_value::NumericVector;
use super::NexusUnits;
use hdf5::{Group, H5Type};

use crate::error::NexusPushError;

/// Implemented for objects who are constructed by a builder
/// i.e. NexusDataset and NexusAttribute instances
pub(in crate::schematic) trait NexusBuildable: Sized {
    type Builder: NexusBuilderBegun;

    fn begin(name: &str) -> Self::Builder;
}

/// Implemented for builders which require input
/// i.e. NexusBuilder with FINISHED = false
pub(in crate::schematic) trait NexusBuilderBegun: Sized {
    type FinshedBuilder: NexusBuilderFinished;

    fn new(name: &str) -> Self;
}

/// Implemented for builders which are ready to complete
/// i.e. NexusBuilder with FINISHED = true
pub(in crate::schematic) trait NexusBuilderFinished {
    type BuildType: NexusBuildable;

    fn finish(self) -> Self::BuildType;
}

/// Implemented for objects which can hold data
/// i.e. NexusBuilder with FINISHED = true
pub(in crate::schematic) trait NexusDataHolder: NexusBuildable {
    type HDF5Type;
    type HDF5Container;
    type ThisError;
}

/// Implemented for objects which can hold data
/// i.e. NexusBuilder with FINISHED = true
pub(super) trait NexusH5InstanceCreatableDataHolder: NexusDataHolder {
    fn create_hdf5_instance(
        &self,
        parent: &Self::HDF5Container,
    ) -> Result<Self::HDF5Type, Self::ThisError>;
}

/// Implemented for objects which can hold data
/// i.e. NexusBuilder with FINISHED = true
pub(super) trait NexusH5CreatableDataHolder: NexusH5InstanceCreatableDataHolder {
    fn create_hdf5(&mut self, parent: &Self::HDF5Container) -> Result<(), Self::ThisError>;
    fn close_hdf5(&mut self);
}

/// Implemented for objects which can hold data
/// i.e. NexusBuilder with FINISHED = true
pub(super) trait NexusDataHolderWithStaticType: NexusDataHolder {
    type DataType: H5Type + Default + Clone;
}

/// Implemented for `NexusDataHolder` objects have mutable scalar data
/// i.e. NexusDataset and NexusAttribute instances with C = NexusDataHolderMutable
pub(in crate::schematic) trait NexusDataHolderScalarMutable:
    NexusDataHolderWithStaticType
{
    fn write_scalar(
        &self,
        parent: &Self::HDF5Container,
        value: Self::DataType,
    ) -> Result<(), Self::ThisError>;

    fn read_scalar(&self, parent: &Self::HDF5Container) -> Result<Self::DataType, Self::ThisError>;
}

/// Implemented for `NexusDataHolder` objects have extendable vector data
/// i.e. NexusDataset and NexusAttribute instances with C = NexusDataHolderResizable
pub(in crate::schematic) trait NexusDataHolderWithSize:
    NexusDataHolder
{
    fn get_size(&self, parent: &Self::HDF5Container) -> Result<usize, Self::ThisError>;
}

/// Implemented for `NexusDataHolder` objects have extendable vector data
/// i.e. NexusDataset and NexusAttribute instances with C = NexusDataHolderResizable
pub(in crate::schematic) trait NexusAppendableDataHolder:
    NexusDataHolderWithStaticType + NexusDataHolderWithSize
{
    fn append(
        &self,
        parent: &Self::HDF5Container,
        values: &[Self::DataType],
    ) -> Result<(), Self::ThisError>;
}

/// Implemented for `NexusDataHolder` objects have extendable vector data
/// i.e. NexusDataset and NexusAttribute instances with C = NexusDataHolderResizable
pub(in crate::schematic) trait NexusNumericAppendableDataHolder:
    NexusDataHolderWithSize
{
    fn append_numerics(
        &self,
        parent: &Self::HDF5Container,
        values: &NumericVector,
    ) -> Result<(), Self::ThisError>;
}

/// Implemented for structs in the `groups` folder which define the HDF5 group structure
pub(in crate::schematic) trait NexusGroupDef: Sized {
    const CLASS_NAME: &'static str;
    type Settings;

    fn new(_settings: &Self::Settings) -> Self;
}

/// Implemented for structs in the `groups` folder which define the HDF5 dataset structure
pub(in crate::schematic) trait NexusDatasetDef: Sized {
    const UNITS: Option<NexusUnits> = None;

    fn new() -> Self;
}

impl NexusDatasetDef for () {
    fn new() -> Self {
        ()
    }
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
