use hdf5::{Attribute, Dataset, Group, H5Type};
use std::marker::PhantomData;

use crate::{
    error::{HDF5Error, NexusAttributeError},
    schematic::H5String,
};

use super::{
    dataholder_class::{
        NexusClassDataHolder, NexusClassFixedDataHolder, NexusClassMutableDataHolder,
    },
    traits::{
        NexusContainerWithAttribute, NexusDataHolder, NexusDataHolderFixed, NexusDataHolderScalarMutable, NexusDataHolderStringMutable, NexusDataHolderWithStaticType, NexusH5CreatableDataHolder, NexusH5InstanceCreatableDataHolder
    },
};

#[derive(Clone)]
pub(crate) struct NexusAttribute<C: NexusClassDataHolder, P : NexusContainerWithAttribute>
{
    name: String,
    class: C,
    attribute: Option<Attribute>,
    phantom: PhantomData<P>
}

pub(crate) type NexusAttributeMut<T,P = Dataset> = NexusAttribute<NexusClassMutableDataHolder<T>,P>;

pub(crate) type NexusAttributeFixed<T,P = Dataset> = NexusAttribute<NexusClassFixedDataHolder<T>,P>;

impl NexusContainerWithAttribute for Dataset {
    fn attribute<T : H5Type>(&self, name: &str) -> Result<Attribute,NexusAttributeError> {
        self.attr(name).or_else(|_| {
            Ok(self.new_attr::<T>()
                .create(name)
                .map_err(HDF5Error::HDF5)?)
        })
    }
}

impl NexusContainerWithAttribute for Group {
    fn attribute<T : H5Type>(&self, name: &str) -> Result<Attribute,NexusAttributeError> {
        self.attr(name).or_else(|_| {
            Ok(self.new_attr::<T>()
                .create(name)
                .map_err(HDF5Error::HDF5)?)
        })
    }
}


impl<C, P> NexusDataHolder for NexusAttribute<C, P>
where
    C: NexusClassDataHolder,
    P: NexusContainerWithAttribute,
{
    type HDF5Type = Attribute;
    type HDF5Container = P;
    type ThisError = NexusAttributeError;
}

impl<C, P> NexusH5CreatableDataHolder for NexusAttribute<C, P>
where
    C: NexusClassDataHolder,
    P: NexusContainerWithAttribute,
    Self: NexusH5InstanceCreatableDataHolder<HDF5Type = Attribute, ThisError = NexusAttributeError>,
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

/*
NexusClassMutableDataHolder
    */
impl<T, P> NexusH5InstanceCreatableDataHolder for NexusAttribute<NexusClassMutableDataHolder<T>, P>
where
    T: H5Type + Clone + Default,
    P: NexusContainerWithAttribute,
{
    fn create_hdf5_instance(
        &self,
        parent: &Self::HDF5Container,
    ) -> Result<Self::HDF5Type, NexusAttributeError> {
        if let Some(ref attribute) = self.attribute {
            Ok(attribute.clone())
        } else {
            let attribute = parent.attribute::<T>(&self.name)?;
            attribute.write_scalar(&self.class.default_value)
                .map_err(HDF5Error::HDF5)?;
            Ok(attribute)
        }
    }
}

impl<T, P> NexusDataHolderWithStaticType for NexusAttribute<NexusClassMutableDataHolder<T>, P>
where
    T: H5Type + Clone + Default,
    P: NexusContainerWithAttribute,
{
    type DataType = T;
}

impl<T, P> NexusDataHolderScalarMutable for NexusAttribute<NexusClassMutableDataHolder<T>, P>
where
    T: H5Type + Clone + Default,
    P: NexusContainerWithAttribute,
    NexusClassMutableDataHolder<T>: NexusClassDataHolder
{
    fn new_with_initial(name: &str, default_value: Self::DataType) -> Self {
        Self {
            name: name.to_string(),
            class: NexusClassMutableDataHolder { default_value },
            attribute: None,
            phantom: Default::default()
        }
    }

    fn write_scalar(
        &self,
        parent: &Self::HDF5Container,
        value: Self::DataType,
    ) -> Result<(), Self::ThisError> {
        let attribute = self.create_hdf5_instance(parent)?;
        Ok(attribute.write_scalar(&value).map_err(HDF5Error::HDF5)?)
    }

    fn read_scalar(&self, parent: &Self::HDF5Container) -> Result<Self::DataType, Self::ThisError> {
        let attribute = self.create_hdf5_instance(parent)?;
        Ok(attribute.read_scalar().map_err(HDF5Error::HDF5)?)
    }
}

impl<P> NexusDataHolderStringMutable for NexusAttribute<NexusClassMutableDataHolder<H5String>, P> where
    P: NexusContainerWithAttribute,
    Self::ThisError: From<HDF5Error>
{
}

/*
NexusClassFixedDataHolder
    */

impl<T,P> NexusH5InstanceCreatableDataHolder for NexusAttribute<NexusClassFixedDataHolder<T>, P>
where
    T: H5Type + Clone + Default,
    P: NexusContainerWithAttribute,
{
    fn create_hdf5_instance(
        &self,
        parent: &Self::HDF5Container,
    ) -> Result<Self::HDF5Type, Self::ThisError> {
        let attribute = parent.attribute::<T>(&self.name)?;
        attribute
            .write_scalar(&self.class.fixed_value)
            .map_err(HDF5Error::HDF5)?;
        Ok(attribute)
    }
}

impl<T,P> NexusDataHolderWithStaticType for NexusAttribute<NexusClassFixedDataHolder<T>, P>
where
    T: H5Type + Clone + Default,
    P: NexusContainerWithAttribute,
{
    type DataType = T;
}

impl<T,P> NexusDataHolderFixed for NexusAttribute<NexusClassFixedDataHolder<T>, P>
where
    T: H5Type + Clone + Default,
    NexusClassFixedDataHolder<T>: NexusClassDataHolder,
    P: NexusContainerWithAttribute,
    Self: NexusH5InstanceCreatableDataHolder<HDF5Type = Attribute, ThisError = NexusAttributeError>
{
    fn new_with_fixed_value(name: &str, fixed_value: Self::DataType) -> Self {
        Self {
            name: name.to_string(),
            class: NexusClassFixedDataHolder { fixed_value },
            attribute: None,
            phantom: Default::default()
        }
    }

    fn write(&self, parent: &Self::HDF5Container) -> Result<(), Self::ThisError> {
        self.create_hdf5_instance(parent)?;
        Ok(())
    }
}
