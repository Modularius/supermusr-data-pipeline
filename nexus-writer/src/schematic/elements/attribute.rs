use std::marker::PhantomData;

use hdf5::{Attribute, Dataset, H5Type};

use super::{
    builder::{NexusBuilder, NexusDataHolderConstant, NexusDataHolderMutable},
    NexusBuildable, NexusBuilderFinished, NexusDataHolder, NexusDataHolderClass,
    NexusDataHolderScalarMutable, NexusError,NexusBuilderBegun
};

impl<T: H5Type + Clone + Default, C: NexusDataHolderClass> NexusBuilderFinished
    for NexusBuilder<C, NexusAttribute<T, C>, true>
where
    NexusAttribute<T, C>: NexusDataHolder,
{
    type BuiltType = NexusAttribute<T, C>;

    fn finish(self) -> NexusAttribute<T, C> {
        NexusAttribute {
            name: self.name,
            class: self.class,
            attribute: None,
            phantom: PhantomData,
        }
    }
}

#[derive(Clone)]
pub(crate) struct NexusAttribute<
    T: H5Type + Clone + Default,
    C: NexusDataHolderClass = NexusDataHolderMutable<T>,
> {
    name: String,
    class: C,
    attribute: Option<Attribute>,
    phantom: PhantomData<T>,
}
pub(crate) type NexusAttributeFixed<T> = NexusAttribute<T, NexusDataHolderConstant<T>>;

impl<T, C> NexusBuildable for NexusAttribute<T, C>
where
    T: H5Type + Clone + Default,
    C: NexusDataHolderClass,
    NexusAttribute<T, C>: NexusDataHolder,
{
    type Builder = NexusBuilder<C, NexusAttribute<T, C>, false>;

    fn begin(name: &str) -> Self::Builder {
        Self::Builder::new(name)
    }
}

impl<T> NexusDataHolder for NexusAttribute<T, NexusDataHolderMutable<T>>
where
    T: H5Type + Clone + Default,
{
    type DataType = T;
    type HDF5Type = Attribute;
    type HDF5Container = Dataset;

    fn create_hdf5(&mut self, parent: &Self::HDF5Container) -> Result<(), NexusError> {
        let attribute = parent.attr(&self.name).or_else(|_| {
            let attribute = parent
                .new_attr::<T>()
                .create(self.name.as_str())
                .map_err(|_| NexusError::Unknown)?;
            attribute
                .write_scalar(&self.class.default_value)
                .map_err(|_| NexusError::Unknown)?;
            Ok(attribute)
        })?;
        self.attribute = Some(attribute);
        Ok(())
    }
    fn close_hdf5(&mut self) {
        self.attribute = None;
    }
}

impl<T> NexusDataHolder for NexusAttribute<T, NexusDataHolderConstant<T>>
where
    T: H5Type + Clone + Default,
{
    type DataType = T;
    type HDF5Type = Attribute;
    type HDF5Container = Dataset;

    fn create_hdf5(&mut self, parent: &Self::HDF5Container) -> Result<(), NexusError> {
        let attribute = parent.attr(&self.name).or_else(|_| {
            let attribute = parent
                .new_attr::<T>()
                .create(self.name.as_str())
                .map_err(|_| NexusError::Unknown)?;
            attribute
                .write_scalar(&self.class.fixed_value)
                .map_err(|_| NexusError::Unknown)?;
            Ok(attribute)
        })?;
        self.attribute = Some(attribute);
        Ok(())
    }
    fn close_hdf5(&mut self) {
        self.attribute = None;
    }
}

impl<T> NexusDataHolderScalarMutable for NexusAttribute<T, NexusDataHolderMutable<T>>
where
    T: H5Type + Clone + Default,
    NexusDataHolderMutable<T>: NexusDataHolderClass,
{
    fn write_scalar(&self, value: Self::DataType) -> Result<(), NexusError> {
        if let Some(attribute) = self.attribute.as_ref() {
            attribute
                .write_scalar(&value)
                .map_err(|_| NexusError::Unknown)?;
        }
        Ok(())
    }

    fn read_scalar(&self) -> Result<Self::DataType, NexusError> {
        self.attribute.as_ref()
            .ok_or(NexusError::Unknown)
            .and_then(|dataset| dataset.read_scalar().map_err(|_| NexusError::Unknown))
    }
}
