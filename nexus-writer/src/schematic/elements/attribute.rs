use std::marker::PhantomData;

use hdf5::{Attribute, Dataset, H5Type};

use crate::error::NexusAttributeError;

use super::{
    builder::{NexusBuilder, NexusDataHolderConstant, NexusDataHolderMutable},
    NexusBuildable, NexusBuilderBegun, NexusBuilderFinished, NexusDataHolder, NexusDataHolderClass,
    NexusDataHolderScalarMutable, NexusTypedDataHolder,
};

impl<T: H5Type + Clone + Default, C: NexusDataHolderClass> NexusBuilderFinished
    for NexusBuilder<C, NexusAttribute<T, C>, true>
where
    NexusAttribute<T, C>: NexusDataHolder,
{
    type BuildType = NexusAttribute<T, C>;

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
pub(in crate::schematic) struct NexusAttribute<
    T: H5Type + Clone + Default,
    C: NexusDataHolderClass = NexusDataHolderMutable<T>,
> {
    name: String,
    class: C,
    attribute: Option<Attribute>,
    phantom: PhantomData<T>,
}

pub(in crate::schematic) type NexusAttributeFixed<T> =
    NexusAttribute<T, NexusDataHolderConstant<T>>;

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
    type HDF5Type = Attribute;
    type HDF5Container = Dataset;
    type ThisError = NexusAttributeError;

    fn create_hdf5_instance(
        &self,
        parent: &Self::HDF5Container,
    ) -> Result<Self::HDF5Type, NexusAttributeError> {
        if let Some(ref attribute) = self.attribute {
            Ok(attribute.clone())
        } else {
            parent.attr(&self.name).or_else(|_| {
                let attribute = parent.new_attr::<T>().create(self.name.as_str())?;
                attribute.write_scalar(&self.class.default_value)?;
                Ok::<_, NexusAttributeError>(attribute)
            })
        }
    }

    fn create_hdf5(&mut self, parent: &Self::HDF5Container) -> Result<(), NexusAttributeError> {
        let attribute = self.create_hdf5_instance(parent)?;
        self.attribute = Some(attribute.clone());
        Ok(())
    }

    fn close_hdf5(&mut self) {
        self.attribute = None;
    }
}

impl<T> NexusTypedDataHolder for NexusAttribute<T, NexusDataHolderMutable<T>>
where
    T: H5Type + Clone + Default,
{
    type DataType = T;
}

impl<T> NexusDataHolder for NexusAttribute<T, NexusDataHolderConstant<T>>
where
    T: H5Type + Clone + Default,
{
    type HDF5Type = Attribute;
    type HDF5Container = Dataset;
    type ThisError = NexusAttributeError;

    fn create_hdf5_instance(
        &self,
        parent: &Self::HDF5Container,
    ) -> Result<Self::HDF5Type, NexusAttributeError> {
        parent.attr(&self.name).or_else(|_| {
            let attribute = parent.new_attr::<T>().create(self.name.as_str())?;
            attribute.write_scalar(&self.class.fixed_value)?;
            Ok::<_, NexusAttributeError>(attribute)
        })
    }

    fn create_hdf5(&mut self, parent: &Self::HDF5Container) -> Result<(), NexusAttributeError> {
        let attribute = self.create_hdf5_instance(parent)?;
        self.attribute = Some(attribute.clone());
        Ok(())
    }

    fn close_hdf5(&mut self) {
        self.attribute = None;
    }
}

impl<T> NexusTypedDataHolder for NexusAttribute<T, NexusDataHolderConstant<T>>
where
    T: H5Type + Clone + Default,
{
    type DataType = T;
}

impl<T> NexusDataHolderScalarMutable for NexusAttribute<T, NexusDataHolderMutable<T>>
where
    T: H5Type + Clone + Default,
    NexusDataHolderMutable<T>: NexusDataHolderClass,
{
    fn write_scalar(
        &self,
        parent: &Self::HDF5Container,
        value: Self::DataType,
    ) -> Result<(), NexusAttributeError> {
        let attribute = self.create_hdf5_instance(parent)?;
        Ok(attribute.write_scalar(&value)?)
    }

    fn read_scalar(
        &self,
        parent: &Self::HDF5Container,
    ) -> Result<Self::DataType, NexusAttributeError> {
        let attribute = self.create_hdf5_instance(parent)?;
        Ok(attribute.read_scalar()?)
    }
}
