use hdf5::{Attribute, Dataset, H5Type};

use crate::{
    error::{HDF5Error, NexusAttributeError},
    schematic::{H5DateTimeString, H5String},
};

use super::{
    dataholder_class::{
        NexusClassDataHolder, NexusClassFixedDataHolder, NexusClassMutableDataHolder,
    },
    traits::{
        NexusDataHolder, NexusDataHolderFixed, NexusDataHolderScalarMutable, NexusDataHolderStringMutable, NexusDataHolderWithStaticType, NexusH5CreatableDataHolder, NexusH5InstanceCreatableDataHolder
    },
};

#[derive(Clone)]
pub(crate) struct NexusAttribute<C: NexusClassDataHolder> {
    name: String,
    class: C,
    attribute: Option<Attribute>,
}

pub(crate) type NexusAttributeMut<T> = NexusAttribute<NexusClassMutableDataHolder<T>>;

pub(crate) type NexusAttributeFixed<T> = NexusAttribute<NexusClassFixedDataHolder<T>>;

impl<C> NexusDataHolder for NexusAttribute<C>
where
    C: NexusClassDataHolder,
{
    type HDF5Type = Attribute;
    type HDF5Container = Dataset;
    type ThisError = NexusAttributeError;
}

impl<C> NexusH5CreatableDataHolder for NexusAttribute<C>
where
    C: NexusClassDataHolder,
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
                let attribute = parent
                    .new_attr::<T>()
                    .create(self.name.as_str())
                    .map_err(HDF5Error::HDF5)?;
                attribute
                    .write_scalar(&self.class.default_value)
                    .map_err(HDF5Error::HDF5)?;
                Ok::<_, NexusAttributeError>(attribute)
            })
        }
    }
}

impl<T> NexusDataHolderWithStaticType for NexusAttribute<NexusClassMutableDataHolder<T>>
where
    T: H5Type + Clone + Default,
{
    type DataType = T;
}

impl<T> NexusDataHolderScalarMutable for NexusAttribute<NexusClassMutableDataHolder<T>>
where
    T: H5Type + Clone + Default,
    NexusClassMutableDataHolder<T>: NexusClassDataHolder,
{
    fn new_with_initial(name: &str, default_value: Self::DataType) -> Self {
        Self {
            name: name.to_string(),
            class: NexusClassMutableDataHolder { default_value },
            attribute: None,
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

impl NexusDataHolderStringMutable for NexusAttribute<NexusClassMutableDataHolder<H5String>> where
    Self::ThisError: From<HDF5Error>
{
}

/*
NexusClassFixedDataHolder
    */

impl<T> NexusH5InstanceCreatableDataHolder for NexusAttribute<NexusClassFixedDataHolder<T>>
where
    T: H5Type + Clone + Default,
{
    fn create_hdf5_instance(
        &self,
        parent: &Self::HDF5Container,
    ) -> Result<Self::HDF5Type, Self::ThisError> {
        parent.attr(&self.name).or_else(|_| {
            let attribute = parent
                .new_attr::<T>()
                .create(self.name.as_str())
                .map_err(HDF5Error::HDF5)?;
            attribute
                .write_scalar(&self.class.fixed_value)
                .map_err(HDF5Error::HDF5)?;
            Ok::<_, Self::ThisError>(attribute)
        })
    }
}

impl<T> NexusDataHolderWithStaticType for NexusAttribute<NexusClassFixedDataHolder<T>>
where
    T: H5Type + Clone + Default,
{
    type DataType = T;
}

impl<T> NexusDataHolderFixed for NexusAttribute<NexusClassFixedDataHolder<T>>
where
    T: H5Type + Clone + Default,
    NexusClassFixedDataHolder<T>: NexusClassDataHolder,
{
    fn new_with_fixed_value(name: &str, fixed_value: Self::DataType) -> Self {
        Self {
            name: name.to_string(),
            class: NexusClassFixedDataHolder { fixed_value },
            attribute: None,
        }
    }
}
