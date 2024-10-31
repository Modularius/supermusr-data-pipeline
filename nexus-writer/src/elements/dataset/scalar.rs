use hdf5::H5Type;

use crate::{
    error::{HDF5Error, NexusDatasetError},
    schematic::H5String,
    elements::{
        dataholder_class::{NexusClassFixedDataHolder, NexusClassMutableDataHolder},
        dataset::NexusDataset,
        traits::{
            NexusDataHolderFixed, NexusDataHolderScalarMutable, NexusDataHolderStringMutable,
            NexusDatasetDef, NexusH5InstanceCreatableDataHolder,
        },
    }
};

/*
NexusClassMutableDataHolder
    */
impl<T, D> NexusH5InstanceCreatableDataHolder for NexusDataset<D, NexusClassMutableDataHolder<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
{
    fn create_hdf5_instance(
        &self,
        parent: &Self::HDF5Container,
    ) -> Result<hdf5::Dataset, NexusDatasetError> {
        parent.dataset(&self.name).or_else(|_| {
            let dataset = parent
                .new_dataset::<T>()
                .create(self.name.as_str())
                .map_err(HDF5Error::HDF5)?;
            dataset
                .write_scalar(&self.class.default_value)
                .map_err(HDF5Error::HDF5)?;
            self.create_units(&dataset)?;
            Ok::<_, NexusDatasetError>(dataset)
        })
    }
}

impl<T, D> NexusDataHolderScalarMutable for NexusDataset<D, NexusClassMutableDataHolder<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
{
    fn new_with_initial(name: &str, default_value: Self::DataType) -> Self {
        Self {
            name: name.to_string(),
            class: NexusClassMutableDataHolder { default_value },
            definition: D::new(),
        }
    }

    fn write_scalar(
        &self,
        parent: &Self::HDF5Container,
        value: Self::DataType,
    ) -> Result<(), NexusDatasetError> {
        let dataset = self.create_hdf5_instance(parent)?;
        Ok(dataset.write_scalar(&value).map_err(HDF5Error::HDF5)?)
    }

    fn read_scalar(&self, parent: &Self::HDF5Container) -> Result<T, NexusDatasetError> {
        let dataset = self.create_hdf5_instance(parent)?;
        Ok(dataset.read_scalar().map_err(HDF5Error::HDF5)?)
    }

    fn mutate<F>(&self, parent: &Self::HDF5Container, f: F) -> Result<(), Self::ThisError>
    where
        F: Fn(&Self::DataType) -> Self::DataType,
    {
        let dataset = self.create_hdf5_instance(parent)?;
        let value = dataset.read_scalar().map_err(HDF5Error::HDF5)?;
        dataset.write_scalar(&f(&value)).map_err(HDF5Error::HDF5)?;
        Ok(())
    }
}

/*
NexusClassMutableDataHolder<H5String>
    */

impl<D: NexusDatasetDef> NexusDataHolderStringMutable
    for NexusDataset<D, NexusClassMutableDataHolder<H5String>>
{
}

/*
NexusClassFixedDataHolder
    */

impl<T, D> NexusH5InstanceCreatableDataHolder for NexusDataset<D, NexusClassFixedDataHolder<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
{
    fn create_hdf5_instance(
        &self,
        parent: &Self::HDF5Container,
    ) -> Result<hdf5::Dataset, NexusDatasetError> {
        parent.dataset(&self.name).or_else(|_| {
            let dataset = parent
                .new_dataset::<T>()
                .create(self.name.as_str())
                .map_err(HDF5Error::HDF5)?;
            dataset
                .write_scalar(&self.class.fixed_value)
                .map_err(HDF5Error::HDF5)?;
            self.create_units(&dataset)?;
            Ok::<_, NexusDatasetError>(dataset)
        })
    }
}

impl<T, D> NexusDataHolderFixed for NexusDataset<D, NexusClassFixedDataHolder<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
{
    fn new_with_fixed_value(name: &str, fixed_value: Self::DataType) -> Self {
        Self {
            name: name.to_string(),
            class: NexusClassFixedDataHolder { fixed_value },
            definition: D::new(),
        }
    }

    fn write(&self, parent: &Self::HDF5Container) -> Result<(), Self::ThisError> {
        let dataset = self.create_hdf5_instance(parent)?;
        self.create_units(&dataset)?;
        Ok(())
    }
}
