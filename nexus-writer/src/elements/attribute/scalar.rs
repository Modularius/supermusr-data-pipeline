use hdf5::H5Type;

use crate::{
    error::{HDF5Error, NexusAttributeError},
    schematic::H5String,
    elements::{
        attribute::NexusAttribute,
        dataholder_class::{
            NexusClassDataHolder, NexusClassFixedDataHolder, NexusClassMutableDataHolder
        },
        traits::{
            NexusContainerWithAttribute, NexusDataHolderFixed,
            NexusDataHolderScalarMutable, NexusDataHolderStringMutable, NexusH5InstanceCreatableDataHolder,
        }
    }
};


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
        parent.attribute::<T,_>(&self.name, |attr| {
            attr
                .write_scalar(&self.class.default_value)
                .map_err(HDF5Error::HDF5)?;
            Ok(attr)
        })
    }
}

impl<T, P> NexusDataHolderScalarMutable for NexusAttribute<NexusClassMutableDataHolder<T>, P>
where
    T: H5Type + Clone + Default,
    P: NexusContainerWithAttribute,
    NexusClassMutableDataHolder<T>: NexusClassDataHolder,
{
    fn new_with_initial(name: &str, default_value: Self::DataType) -> Self {
        Self {
            name: name.to_string(),
            class: NexusClassMutableDataHolder { default_value },
            phantom: Default::default(),
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

    fn mutate<F>(&self, parent: &Self::HDF5Container, f: F) -> Result<(), Self::ThisError>
    where
        F: Fn(&Self::DataType) -> Self::DataType,
    {
        let attribute = self.create_hdf5_instance(parent)?;
        let value = attribute.read_scalar().map_err(HDF5Error::HDF5)?;
        attribute
            .write_scalar(&f(&value))
            .map_err(HDF5Error::HDF5)?;
        Ok(())
    }
}

impl<P: NexusContainerWithAttribute> NexusDataHolderStringMutable
    for NexusAttribute<NexusClassMutableDataHolder<H5String>, P>
{
}

/*
NexusClassFixedDataHolder
    */

impl<T, P> NexusH5InstanceCreatableDataHolder for NexusAttribute<NexusClassFixedDataHolder<T>, P>
where
    T: H5Type + Clone + Default,
    P: NexusContainerWithAttribute,
{
    fn create_hdf5_instance(
        &self,
        parent: &Self::HDF5Container,
    ) -> Result<Self::HDF5Type, Self::ThisError> {
        parent.attribute::<T,_>(&self.name, |attr| {
            attr.write_scalar(&self.class.fixed_value)
                .map_err(HDF5Error::HDF5)?;
            Ok(attr)
        })
    }
}

impl<T, P> NexusDataHolderFixed for NexusAttribute<NexusClassFixedDataHolder<T>, P>
where
    T: H5Type + Clone + Default,
    P: NexusContainerWithAttribute,
{
    fn new_with_fixed_value(name: &str, fixed_value: Self::DataType) -> Self {
        Self {
            name: name.to_string(),
            class: NexusClassFixedDataHolder { fixed_value },
            phantom: Default::default(),
        }
    }

    fn write(&self, parent: &Self::HDF5Container) -> Result<(), Self::ThisError> {
        self.create_hdf5_instance(parent)?;
        Ok(())
    }
}
