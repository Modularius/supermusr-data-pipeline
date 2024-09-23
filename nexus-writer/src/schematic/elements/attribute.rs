use std::marker::PhantomData;

use hdf5::{Attribute, Dataset, H5Type};

use crate::error::NexusAttributeError;

use super::{
    builder::NexusBuilder,
    dataholder_class::{
        NexusClassDataHolder, NexusClassFixedDataHolder, NexusClassMutableDataHolder,
        NexusClassWithStaticDataType,
    },
    traits::{
        NexusBuildable, NexusBuilderBegun, NexusBuilderFinished, NexusDataHolder,
        NexusDataHolderScalarMutable, NexusDataHolderWithStaticType, NexusH5CreatableDataHolder,
        NexusH5InstanceCreatableDataHolder,
    },
};

impl<C: NexusClassDataHolder> NexusBuilderFinished for NexusBuilder<C, NexusAttribute<C>, true>
where
    NexusAttribute<C>: NexusDataHolder,
{
    type BuildType = NexusAttribute<C>;

    fn finish(self) -> NexusAttribute<C> {
        NexusAttribute {
            name: self.name,
            class: self.class,
            attribute: None,
        }
    }
}

#[derive(Clone)]
pub(in crate::schematic) struct NexusAttribute<C: NexusClassDataHolder> {
    name: String,
    class: C,
    attribute: Option<Attribute>,
}

pub(in crate::schematic) type NexusAttributeMut<T> = NexusAttribute<NexusClassMutableDataHolder<T>>;

pub(in crate::schematic) type NexusAttributeFixed<T> = NexusAttribute<NexusClassFixedDataHolder<T>>;

impl<C> NexusBuildable for NexusAttribute<C>
where
    C: NexusClassDataHolder,
{
    type Builder = NexusBuilder<C, NexusAttribute<C>, false>;

    fn begin(name: &str) -> Self::Builder {
        Self::Builder::new(name)
    }
}

impl<C> NexusDataHolder for NexusAttribute<C>
where
    C: NexusClassDataHolder,
{
    type HDF5Type = Attribute;
    type HDF5Container = Dataset;
    type ThisError = NexusAttributeError;
}

impl<T> NexusH5InstanceCreatableDataHolder for NexusAttribute<NexusClassMutableDataHolder<T>>
where
    T: H5Type + Clone + Default,
{
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
}
impl<C> NexusH5CreatableDataHolder for NexusAttribute<C>
where
    C: NexusClassDataHolder,
{
    fn create_hdf5(&mut self, parent: &Self::HDF5Container) -> Result<(), NexusAttributeError> {
        let attribute = self.create_hdf5_instance(parent)?;
        self.attribute = Some(attribute.clone());
        Ok(())
    }

    fn close_hdf5(&mut self) {
        self.attribute = None;
    }
}

impl<T, C> NexusDataHolderWithStaticType for NexusAttribute<C>
where
    T: H5Type + Clone + Default,
    C: NexusClassWithStaticDataType<T>,
{
    type DataType = T;
}

impl<T> NexusH5InstanceCreatableDataHolder for NexusAttribute<NexusClassFixedDataHolder<T>>
where
    T: H5Type + Clone + Default,
{
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
}

impl<T> NexusDataHolderScalarMutable for NexusAttribute<NexusClassMutableDataHolder<T>>
where
    T: H5Type + Clone + Default,
    NexusClassMutableDataHolder<T>: NexusClassDataHolder,
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
