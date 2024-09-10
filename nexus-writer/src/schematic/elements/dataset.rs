use std::marker::PhantomData;

use hdf5::{Dataset, Group, H5Type};
use ndarray::s;

use super::{
    builder::{NexusBuilder, NexusDataHolderMutable, NexusDataHolderResizable}, NexusBuildable, NexusBuilderFinished, NexusDataHolder, NexusDataHolderAppendable, NexusDataHolderClass, NexusDataHolderScalarMutable, NexusDatasetDef, NexusError
};

impl<T: H5Type + Clone + Default, C: NexusDataHolderClass, D: NexusDatasetDef> NexusBuilderFinished<NexusDataset<T, D, C>>
    for NexusBuilder<C, NexusDataset<T, D, C>, true>
{
    fn finish(self) -> Result<NexusDataset<T, D, C>, super::NexusError> {
        Ok(NexusDataset {
            name: self.name,
            class: self.class,
            dataset: None,
            definition: D::new(),
        })
    }
}

pub(crate) struct NexusDataset<T: H5Type + Clone + Default, D: NexusDatasetDef, C: NexusDataHolderClass> {
    name: String,
    class: C,
    dataset: Option<Dataset>,
    definition: D,
}

impl<T: H5Type + Clone + Default, D: NexusDatasetDef, C: NexusDataHolderClass> NexusBuildable for NexusDataset<T, D, C> {
    type Builder = NexusBuilder<C, NexusDataset<T, D, C>, false>;

    fn begin(name: &str) -> Self::Builder {
        NexusBuilder {
            name: name.to_string(),
            class: C::default(),
            phantom: PhantomData,
        }
    }
}

impl<D: NexusDatasetDef, T: H5Type + Clone + Default> NexusDataHolder
    for NexusDataset<T, D, NexusDataHolderMutable<T>>
{
    type DataType = T;
    type HDF5Type = Dataset;
    type HDF5Container = Group;

    fn create_hdf5(
        &mut self,
        parent: &Self::HDF5Container,
    ) -> Result<Self::HDF5Type, NexusError> {
        self.dataset = Some(
            parent.dataset(&self.name).or_else(||{
                let dataset = parent
                    .new_dataset::<T>()
                    .create(self.name)
                    .map_err(|_|NexusError)?;
                dataset.write_scalar(&self.class.default_value).map_err(|_|NexusError)?
            })
        );
    }
    fn close_hdf5(&mut self) {
        self.dataset = None;
    }
}

impl<D: NexusDatasetDef, T: H5Type + Clone + Default> NexusDataHolderScalarMutable
    for NexusDataset<T, D, NexusDataHolderMutable<T>> {
    fn write_scalar(&self, value: Self::DataType) -> Result<(), NexusError> {
        if let Some(dataset) = self.dataset {
            dataset.write_scalar(&value)?;
        }
        Ok(())
    }
    
    fn read_scalar(&self) -> Result<Self::DataType, NexusError> {
        self.dataset.ok_or(NexusError).map(|dataset|dataset.read_scalar()?)
    }
}

impl<D: NexusDatasetDef, T: H5Type + Clone + Default> NexusDataHolderAppendable
    for NexusDataset<T, D, NexusDataHolderResizable<T>> {
    
    fn append(&self, values: &[Self::DataType]) -> Result<(), NexusError> {
        self.dataset
            .as_ref()
            .ok_or_else(NexusError)
            .and_then(|dataset| {
                let size = dataset.size();
                let next_values_slice = s![size..(size + values.len())];
                dataset.resize(size + values.len())?;
                dataset.write_slice(values, next_values_slice)?;
                Ok(())
            })
    }
    
    fn get_size(&self) -> Result<usize,NexusError> {
        self.dataset.ok_or(NexusError).map(|dataset|dataset.size())
    }
}