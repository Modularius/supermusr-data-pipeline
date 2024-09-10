use hdf5::H5Type;
use thiserror::Error;

pub(crate) mod attribute;
pub(crate) mod builder;
pub(crate) mod dataset;
pub(crate) mod group;

#[derive(Debug, Error)]
pub(crate) struct NexusError;

pub(crate) trait NexusBuildable {
    type Builder: NexusBuilderBegun<Self>;

    fn begin(name: &str) -> Self::Builder;
}

pub(crate) trait NexusBuilderBegun<B>
where
    B: NexusBuildable,
{
    type FinshedBuilder: NexusBuilderFinished<B>;
}

pub(crate) trait NexusBuilderFinished<B>
where
    B: NexusBuildable,
{
    fn finish(self) -> Result<B, NexusError>;
}

pub(crate) trait NexusDataHolder: NexusBuildable {
    type DataType: H5Type;

    type HDF5Type;
    type HDF5Container;

    fn create_hdf5(&self, parent: &Self::HDF5Container) -> Result<Self::HDF5Type, NexusError>;
    fn close_hdf5(&mut self);
}

pub(crate) trait NexusDataHolderScalarMutable: NexusDataHolder {
    fn write_scalar(&self, value: Self::DataType) -> Result<(), NexusError>;
    fn read_scalar(&self) -> Result<Self::DataType, NexusError>;
}

pub(crate) trait NexusDataHolderAppendable: NexusDataHolder {
    fn append(&self, values: &[Self::DataType]) -> Result<(), NexusError>;
    fn get_size(&self) -> Result<usize,NexusError>;
}

pub(crate) trait NexusDataHolderClass: Default + Clone {}

pub(crate) trait NexusGroupDef {
    const CLASS_NAME: &'static str;

    fn new() -> Result<Self, NexusError>;
}

pub(crate) trait NexusDatasetDef {
    fn new() -> Result<Self, NexusError>;
}
