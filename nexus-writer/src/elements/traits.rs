use std::str::FromStr;

use super::log_value::NumericVector;
use super::NexusUnits;
use chrono::{DateTime, Utc};
use hdf5::{types::TypeDescriptor, Group, H5Type};

use crate::{
    error::{HDF5Error, NexusPushError},
    schematic::{H5DateTimeString, H5String},
};

/// Implemented for objects who are constructed by a builder
/// i.e. NexusDataset and NexusAttribute instances
pub(crate) trait NexusBuildable: Sized {
    type Builder: NexusBuilderBegun;

    fn begin(name: &str) -> Self::Builder;
}

/// Implemented for builders which require input
/// i.e. NexusBuilder with FINISHED = false
pub(crate) trait NexusBuilderBegun: Sized {
    type FinshedBuilder: NexusBuilderFinished;

    fn new(name: &str) -> Self;
}

/// Implemented for builders which are ready to complete
/// i.e. NexusBuilder with FINISHED = true
pub(crate) trait NexusBuilderFinished {
    type BuildType: NexusBuildable;

    fn finish(self) -> Self::BuildType;
}

/// Implemented for objects which can hold data
/// i.e. NexusBuilder with FINISHED = true
pub(crate) trait NexusDataHolder {
    type HDF5Type;
    type HDF5Container;
    type ThisError;
}

/// Implemented for objects which can hold data
/// i.e. NexusBuilder with FINISHED = true
pub(crate) trait NexusH5InstanceCreatableDataHolder: NexusDataHolder {
    fn create_hdf5_instance(
        &self,
        parent: &Self::HDF5Container,
    ) -> Result<Self::HDF5Type, Self::ThisError>;
}

/// Implemented for objects which can hold data
/// i.e. NexusBuilder with FINISHED = true
pub(crate) trait NexusH5CreatableDataHolder: NexusH5InstanceCreatableDataHolder {
    fn create_hdf5(&mut self, parent: &Self::HDF5Container) -> Result<(), Self::ThisError>;
    fn close_hdf5(&mut self);
}

/// Implemented for objects which can hold data
/// i.e. NexusBuilder with FINISHED = true
pub(crate) trait NexusDataHolderWithStaticType: NexusDataHolder {
    type DataType: H5Type + Default + Clone;
}

/// Implemented for `NexusDataHolder` objects have mutable scalar data
/// i.e. NexusDataset and NexusAttribute instances with C = NexusDataHolderMutable
pub(crate) trait NexusDataHolderScalarMutable:
    NexusDataHolderWithStaticType + Sized
{
    fn new_with_initial(name: &str, default: Self::DataType) -> Self;

    fn new_with_default(name: &str) -> Self {
        Self::new_with_initial(name, Default::default())
    }

    fn write_scalar(
        &self,
        parent: &Self::HDF5Container,
        value: Self::DataType,
    ) -> Result<(), Self::ThisError>;

    fn read_scalar(&self, parent: &Self::HDF5Container) -> Result<Self::DataType, Self::ThisError>;
}

pub(crate) trait NexusDataHolderFixed: NexusDataHolderWithStaticType {
    fn new_with_fixed_value(name: &str, fixed_value: Self::DataType) -> Self;
}

pub(crate) trait NexusDataHolderStringMutable:
    NexusDataHolderScalarMutable + NexusDataHolderWithStaticType<DataType = H5String>
where
    Self::ThisError: From<HDF5Error>,
{
    fn write_string(
        &self,
        parent: &Self::HDF5Container,
        value: &str,
    ) -> Result<(), Self::ThisError> {
        self.write_scalar(parent, value.parse().map_err(HDF5Error::HDF5String)?)
    }
}

pub(crate) trait NexusDataHolderDateTimeMutable:
    NexusDataHolderScalarMutable + NexusDataHolderWithStaticType<DataType = H5DateTimeString>
where
    Self::ThisError: From<chrono::ParseError> + From<HDF5Error>,
{
    fn write_datetime(
        &self,
        parent: &Self::HDF5Container,
        value: &DateTime<Utc>,
    ) -> Result<(), Self::ThisError> {
        self.write_scalar(
            parent,
            value.to_rfc3339().parse().map_err(HDF5Error::HDF5String)?,
        )?;
        Ok(())
    }

    fn read_datetime(
        &self,
        parent: &Self::HDF5Container,
    ) -> Result<DateTime<Utc>, Self::ThisError> {
        Ok(DateTime::<Utc>::from_str(
            self.read_scalar(parent)?.as_str(),
        )?)
    }
}

/// Implemented for `NexusDataHolder` objects have extendable vector data
/// i.e. NexusDataset and NexusAttribute instances with C = NexusDataHolderResizable
pub(crate) trait NexusDataHolderWithSize: NexusDataHolder {
    fn get_size(&self, parent: &Self::HDF5Container) -> Result<usize, Self::ThisError>;
}

/// Implemented for `NexusDataHolder` objects have extendable vector data
/// i.e. NexusDataset and NexusAttribute instances with C = NexusDataHolderResizable
pub(crate) trait NexusAppendableDataHolder:
    NexusDataHolderWithStaticType + NexusDataHolderWithSize + Sized
{
    fn new_with_initial_size(
        name: &str,
        default_value: Self::DataType,
        default_size: usize,
        chunk_size: usize,
    ) -> Self;

    fn new_appendable_with_default(name: &str, chunk_size: usize) -> Self {
        Self::new_with_initial_size(name, Default::default(), Default::default(), chunk_size)
    }

    fn append(
        &self,
        parent: &Self::HDF5Container,
        values: &[Self::DataType],
    ) -> Result<(), Self::ThisError>;
}

/// Implemented for `NexusDataHolder` objects have extendable vector data
/// i.e. NexusDataset and NexusAttribute instances with C = NexusDataHolderResizable
pub(crate) trait NexusNumericAppendableDataHolder: NexusDataHolderWithSize {
    fn new(name: &str, chunk_size: usize) -> Self;

    fn try_set_type(&mut self, type_desc: TypeDescriptor) -> Result<(), Self::ThisError>;

    fn append_numerics(
        &self,
        parent: &Self::HDF5Container,
        values: &NumericVector,
    ) -> Result<(), Self::ThisError>;
}

/// Implemented for structs in the `groups` folder which define the HDF5 group structure
pub(crate) trait NexusGroupDef: Sized {
    const CLASS_NAME: &'static str;
    type Settings;

    fn new(_settings: &Self::Settings) -> Self;
}

/// Implemented for structs in the `groups` folder which define the HDF5 dataset structure
pub(crate) trait NexusDatasetDef: Sized {
    const UNITS: Option<NexusUnits> = None;

    fn new() -> Self;
}

impl NexusDatasetDef for () {
    fn new() -> Self {}
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
