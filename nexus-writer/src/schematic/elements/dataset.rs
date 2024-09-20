use hdf5::{types::TypeDescriptor, Dataset, Group, H5Type, SimpleExtents};
use ndarray::s;
use std::marker::PhantomData;
use thiserror::Error;

use crate::error::{NexusDatasetError, NexusPushError};

use super::{
    attribute::NexusAttribute,
    builder::{
        NexusBuilder, NexusDataHolderConstant, NexusDataHolderMutable, NexusDataHolderResizable, NexusLogValueResizable,
    },
    NexusBuildable, NexusBuilderBegun, NexusBuilderFinished, NexusDataHolder,
    NexusDataHolderAppendable, NexusDataHolderClass, NexusDataHolderScalarMutable, NexusDatasetDef,
    NexusHandleMessage, NexusHandleMessageWithContext, NexusPushMessage,
    NexusPushMessageWithContext, NexusTypedDataHolder,
};

impl<C, D> NexusBuilderFinished for NexusBuilder<C, NexusDataset<D, C>, true>
where
    C: NexusDataHolderClass,
    D: NexusDatasetDef,
    NexusDataset<D, C>: NexusDataHolder,
{
    type BuildType = NexusDataset<D, C>;

    fn finish(self) -> NexusDataset<D, C> {
        NexusDataset {
            name: self.name,
            class: self.class,
            dataset: None,
            definition: D::new(),
        }
    }
}

#[derive(Clone,Default)]
pub(in crate::schematic) struct NexusDataset<
    D: NexusDatasetDef,
    C: NexusDataHolderClass,
> {
    name: String,
    class: C,
    dataset: Option<Dataset>,
    definition: D,
}

impl<D, C> NexusDataset<D, C>
where
    D: NexusDatasetDef,
    C: NexusDataHolderClass,
{
    pub(crate) fn attribute<F, T2, C2>(&self, f: F) -> &NexusAttribute<T2, C2>
    where
        F: Fn(&D) -> &NexusAttribute<T2, C2>,
        T2: H5Type + Clone + Default,
        C2: NexusDataHolderClass,
    {
        f(&self.definition)
    }
}

pub(in crate::schematic) type NexusDatasetMut<T, D = ()> =
    NexusDataset<D, NexusDataHolderMutable<T>>;

pub(in crate::schematic) type NexusDatasetFixed<T, D = ()> =
    NexusDataset<D, NexusDataHolderConstant<T>>;

pub(in crate::schematic) type NexusDatasetResize<T, D = ()> =
    NexusDataset<D, NexusDataHolderResizable<T>>;

pub(in crate::schematic) type NexusLogValueDatasetResize<D = ()> =
    NexusDataset<D, NexusLogValueResizable>;

impl<D, C> NexusBuildable for NexusDataset<D, C>
where
    D: NexusDatasetDef,
    C: NexusDataHolderClass,
    NexusDataset<D, C>: NexusDataHolder,
{
    type Builder = NexusBuilder<C, NexusDataset<D, C>, false>;

    fn begin(name: &str) -> Self::Builder {
        Self::Builder::new(name)
    }
}

impl<T, D> NexusDataHolder for NexusDataset<D, NexusDataHolderMutable<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
    NexusDataHolderMutable<T>: NexusDataHolderClass,
{
    type HDF5Type = Dataset;
    type HDF5Container = Group;
    type ThisError = NexusDatasetError;

    fn create_hdf5_instance(
        &self,
        parent: &Self::HDF5Container,
    ) -> Result<hdf5::Dataset, NexusDatasetError> {
        Ok(parent.dataset(&self.name).or_else(|_| {
            let dataset = parent.new_dataset::<T>().create(self.name.as_str())?;
            dataset.write_scalar(&self.class.default_value)?;
            Ok::<_, NexusDatasetError>(dataset)
        })?)
    }

    fn create_hdf5(&mut self, parent: &Self::HDF5Container) -> Result<(), NexusDatasetError> {
        let dataset = self.create_hdf5_instance(parent)?;
        self.dataset = Some(dataset.clone());
        Ok(())
    }

    fn close_hdf5(&mut self) {
        self.dataset = None;
    }
}

impl<T, D> NexusTypedDataHolder for NexusDataset<D, NexusDataHolderMutable<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
    NexusDataHolderMutable<T>: NexusDataHolderClass,
{
    type DataType = T;
}

impl<T, D> NexusDataHolder for NexusDataset<D, NexusDataHolderConstant<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
{
    type HDF5Type = Dataset;
    type HDF5Container = Group;
    type ThisError = NexusDatasetError;

    fn create_hdf5_instance(
        &self,
        parent: &Self::HDF5Container,
    ) -> Result<hdf5::Dataset, NexusDatasetError> {
        if let Some(ref dataset) = self.dataset {
            Ok(dataset.clone())
        } else {
            parent.dataset(&self.name).or_else(|_| {
                let dataset = parent.new_dataset::<T>().create(self.name.as_str())?;
                dataset.write_scalar(&self.class.fixed_value)?;
                Ok::<_, NexusDatasetError>(dataset)
            })
        }
    }

    fn create_hdf5(&mut self, parent: &Self::HDF5Container) -> Result<(), NexusDatasetError> {
        let dataset = self.create_hdf5_instance(parent)?;
        self.dataset = Some(dataset.clone());
        Ok(())
    }

    fn close_hdf5(&mut self) {
        self.dataset = None;
    }
}

impl<T, D> NexusTypedDataHolder for NexusDataset<D, NexusDataHolderConstant<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
{
    type DataType = T;
}

impl<T, D> NexusDataHolder for NexusDataset<D, NexusDataHolderResizable<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
{
    type HDF5Type = Dataset;
    type HDF5Container = Group;
    type ThisError = NexusDatasetError;

    fn create_hdf5_instance(
        &self,
        parent: &Self::HDF5Container,
    ) -> Result<hdf5::Dataset, NexusDatasetError> {
        if let Some(ref dataset) = self.dataset {
            Ok(dataset.clone())
        } else {
            parent.dataset(&self.name).or_else(|_| {
                let dataset = parent
                    .new_dataset::<T>()
                    .shape(SimpleExtents::resizable(vec![self.class.default_size]))
                    .chunk(vec![self.class.chunk_size])
                    .create(self.name.as_str())?;
                dataset.write_slice(
                    &vec![self.class.default_value.clone(); self.class.default_size],
                    s![0..self.class.default_size],
                )?;
                Ok::<_, NexusDatasetError>(dataset)
            })
        }
    }

    fn create_hdf5(&mut self, parent: &Self::HDF5Container) -> Result<(), NexusDatasetError> {
        let dataset = self.create_hdf5_instance(parent)?;
        self.dataset = Some(dataset.clone());
        Ok(())
    }

    fn close_hdf5(&mut self) {
        self.dataset = None;
    }
}

impl<T, D> NexusTypedDataHolder for NexusDataset<D, NexusDataHolderResizable<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
{
    type DataType = T;
}




impl<D> NexusDataHolder for NexusDataset<D, NexusLogValueResizable>
where
    D: NexusDatasetDef,
{
    type HDF5Type = Dataset;
    type HDF5Container = Group;
    type ThisError = NexusDatasetError;

    fn create_hdf5_instance(
        &self,
        parent: &Self::HDF5Container,
    ) -> Result<hdf5::Dataset, NexusDatasetError> {
        if let Some(ref dataset) = self.dataset {
            Ok(dataset.clone())
        } else {
            parent.dataset(&self.name).or_else(|_| {
                let dataset = parent
                    .new_dataset_builder()
                    .chunk(vec![self.class.chunk_size])
                    .with_data_as(&Vec::<u32>::default(), &self.class.type_desc)
                    .create(self.name.as_str())?;
                Ok::<_, NexusDatasetError>(dataset)
            })
        }
    }

    fn create_hdf5(&mut self, parent: &Self::HDF5Container) -> Result<(), NexusDatasetError> {
        let dataset = self.create_hdf5_instance(parent)?;
        self.dataset = Some(dataset.clone());
        Ok(())
    }

    fn close_hdf5(&mut self) {
        self.dataset = None;
    }
}

impl<T, D> NexusDataHolderScalarMutable for NexusDataset<D, NexusDataHolderMutable<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
    NexusDataHolderMutable<T>: NexusDataHolderClass,
{
    fn write_scalar(
        &self,
        parent: &Self::HDF5Container,
        value: Self::DataType,
    ) -> Result<(), NexusDatasetError> {
        let dataset = self.create_hdf5_instance(parent)?;
        Ok(dataset.write_scalar(&value)?)
    }

    fn read_scalar(&self, parent: &Self::HDF5Container) -> Result<T, NexusDatasetError> {
        let dataset = self.create_hdf5_instance(parent)?;
        Ok(dataset.read_scalar()?)
    }
}

impl<D: NexusDatasetDef, T: H5Type + Clone + Default> NexusDataHolderAppendable
    for NexusDataset<D, NexusDataHolderResizable<T>>
{
    fn append(
        &self,
        parent: &Self::HDF5Container,
        values: &[Self::DataType],
    ) -> Result<(), NexusDatasetError> {
        let dataset = self.create_hdf5_instance(parent)?;
        let size = dataset.size();
        let next_values_slice = s![size..(size + values.len())];
        dataset.resize(size + values.len())?;
        Ok(dataset.write_slice(values, next_values_slice)?)
    }

    fn get_size(&self, parent: &Self::HDF5Container) -> Result<usize, NexusDatasetError> {
        let dataset = self.create_hdf5_instance(parent)?;
        Ok(dataset.size())
    }
}

impl<D, C, M, R> NexusPushMessage<M, Group, R> for NexusDataset<D, C>
where
    D: NexusDatasetDef + NexusHandleMessage<M, Dataset, R>,
    C: NexusDataHolderClass,
    NexusDataset<D, C>:
        NexusDataHolder<HDF5Type = Dataset, HDF5Container = Group, ThisError = NexusDatasetError>,
{
    fn push_message(&mut self, message: &M, parent: &Group) -> Result<R, NexusPushError> {
        let dataset = self.create_hdf5_instance(parent)?;
        let ret = self.definition.handle_message(message, &dataset)?;
        Ok(ret)
    }
}

impl<D, C, M, Ctxt, R> NexusPushMessageWithContext<M, Group, R> for NexusDataset<D, C>
where
    D: NexusDatasetDef + NexusHandleMessageWithContext<M, Dataset, R, Context = Ctxt>,
    C: NexusDataHolderClass,
    NexusDataset<D, C>:
        NexusDataHolder<HDF5Type = Dataset, HDF5Container = Group, ThisError = NexusDatasetError>,
{
    type Context = Ctxt;

    fn push_message_with_context(
        &mut self,
        message: &M,
        parent: &Group,
        context: &mut Self::Context,
    ) -> Result<R, NexusPushError> {
        let parent = self.create_hdf5_instance(parent)?;
        let ret = self
            .definition
            .handle_message_with_context(message, &parent, context)?;
        Ok(ret)
    }
}
