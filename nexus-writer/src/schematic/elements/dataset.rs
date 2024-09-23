use hdf5::{
    types::TypeDescriptor,
    Dataset, Group, H5Type, SimpleExtents,
};
use ndarray::s;

use crate::error::{NexusDatasetError, NexusLogValueError, NexusPushError};

use super::{
    attribute::NexusAttribute, builder::{
        NexusBuilder, NexusClassAppendableDataHolder, NexusClassDataHolderWithSize, NexusClassFixedDataHolder, NexusClassMutableDataHolder, NexusLogValueResizable
    }, log_value::VectorOfScalars, NexusBuildable, NexusBuilderBegun, NexusBuilderFinished, NexusDataHolder, NexusDataHolderAppendable, NexusClassDataHolder, NexusDataHolderScalarMutable, NexusDataHolderSizable, NexusDatasetDef, NexusDatasetDefWithTypeDesciptor, NexusHandleMessage, NexusHandleMessageWithContext, NexusLogValueDataHolderAppendable, NexusPushMessage, NexusPushMessageWithContext, NexusTypedDataHolder
};

impl<C, D> NexusBuilderFinished for NexusBuilder<C, NexusDataset<D, C>, true>
where
    C: NexusClassDataHolder,
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

#[derive(Clone, Default)]
pub(in crate::schematic) struct NexusDataset<D: NexusDatasetDef, C: NexusClassDataHolder> {
    name: String,
    class: C,
    dataset: Option<Dataset>,
    definition: D,
}

impl<D, C> NexusDataset<D, C>
where
    D: NexusDatasetDef,
    C: NexusClassDataHolder,
{
    pub(crate) fn attribute<F, T2, C2>(&self, f: F) -> &NexusAttribute<T2, C2>
    where
        F: Fn(&D) -> &NexusAttribute<T2, C2>,
        T2: H5Type + Clone + Default,
        C2: NexusClassDataHolder,
    {
        f(&self.definition)
    }
}

pub(in crate::schematic) type NexusDatasetMut<T, D = ()> =
    NexusDataset<D, NexusClassMutableDataHolder<T>>;

pub(in crate::schematic) type NexusDatasetFixed<T, D = ()> =
    NexusDataset<D, NexusClassFixedDataHolder<T>>;

pub(in crate::schematic) type NexusDatasetResize<T, D = ()> =
    NexusDataset<D, NexusClassAppendableDataHolder<T>>;

pub(in crate::schematic) type NexusLogValueDatasetResize<D = ()> =
    NexusDataset<D, NexusLogValueResizable>;

impl<D, C> NexusBuildable for NexusDataset<D, C>
where
    D: NexusDatasetDef,
    C: NexusClassDataHolder,
    NexusDataset<D, C>: NexusDataHolder,
{
    type Builder = NexusBuilder<C, NexusDataset<D, C>, false>;

    fn begin(name: &str) -> Self::Builder {
        Self::Builder::new(name)
    }
}

impl<T, D> NexusDataHolder for NexusDataset<D, NexusClassMutableDataHolder<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
    NexusClassMutableDataHolder<T>: NexusClassDataHolder,
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

impl<T, D> NexusTypedDataHolder for NexusDataset<D, NexusClassMutableDataHolder<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
    NexusClassMutableDataHolder<T>: NexusClassDataHolder,
{
    type DataType = T;
}

impl<T, D> NexusDataHolder for NexusDataset<D, NexusClassFixedDataHolder<T>>
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

impl<T, D> NexusTypedDataHolder for NexusDataset<D, NexusClassFixedDataHolder<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
{
    type DataType = T;
}

impl<T, D> NexusDataHolder for NexusDataset<D, NexusClassAppendableDataHolder<T>>
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

impl<T, D> NexusTypedDataHolder for NexusDataset<D, NexusClassAppendableDataHolder<T>>
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
    type HDF5Container = (Group,TypeDescriptor);
    type ThisError = NexusDatasetError;

    fn create_hdf5_instance(
        &self,
        &(parent, type_desc): &Self::HDF5Container,
    ) -> Result<hdf5::Dataset, NexusDatasetError> {
        if let Some(ref dataset) = self.dataset {
            Ok(dataset.clone())
        } else {
            parent.dataset(&self.name).or_else(|_| {
                let builder = parent
                    .new_dataset_builder()
                    .chunk(vec![self.class.chunk_size]);
                let dataset = match type_desc {
                    TypeDescriptor::Integer(int_size) => match int_size {
                        hdf5::types::IntSize::U1 => builder
                            .with_data(&[i8::default(); 0])
                            .create(self.name.as_str()),
                        hdf5::types::IntSize::U2 => builder
                            .with_data(&[i16::default(); 0])
                            .create(self.name.as_str()),
                        hdf5::types::IntSize::U4 => builder
                            .with_data(&[i32::default(); 0])
                            .create(self.name.as_str()),
                        hdf5::types::IntSize::U8 => builder
                            .with_data(&[i64::default(); 0])
                            .create(self.name.as_str()),
                    },
                    TypeDescriptor::Unsigned(int_size) => match int_size {
                        hdf5::types::IntSize::U1 => builder
                            .with_data(&[u8::default(); 0])
                            .create(self.name.as_str()),
                        hdf5::types::IntSize::U2 => builder
                            .with_data(&[u16::default(); 0])
                            .create(self.name.as_str()),
                        hdf5::types::IntSize::U4 => builder
                            .with_data(&[u32::default(); 0])
                            .create(self.name.as_str()),
                        hdf5::types::IntSize::U8 => builder
                            .with_data(&[u64::default(); 0])
                            .create(self.name.as_str()),
                    },
                    TypeDescriptor::Float(float_size) => match float_size {
                        hdf5::types::FloatSize::U4 => builder
                            .with_data(&[f32::default(); 0])
                            .create(self.name.as_str()),
                        hdf5::types::FloatSize::U8 => builder
                            .with_data(&[f64::default(); 0])
                            .create(self.name.as_str()),
                    },
                    _ => Err(hdf5::Error::Internal(Default::default())),
                }?;
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

impl<T, D> NexusDataHolderScalarMutable for NexusDataset<D, NexusClassMutableDataHolder<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
    NexusClassMutableDataHolder<T>: NexusClassDataHolder,
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

impl<D: NexusDatasetDef, C: NexusClassDataHolderWithSize> NexusDataHolderSizable
    for NexusDataset<D, C>
    where NexusDataset<D, C> : NexusDataHolder<HDF5Type = Dataset>
{
    fn get_size(&self, parent: &Self::HDF5Container) -> Result<usize, Self::ThisError> {
        let dataset = self.create_hdf5_instance(parent)?;
        Ok(dataset.size())
    }
}

impl<D: NexusDatasetDef, T: H5Type + Clone + Default> NexusDataHolderAppendable
    for NexusDataset<D, NexusClassAppendableDataHolder<T>>
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
}

impl<D: NexusDatasetDefWithTypeDesciptor> NexusLogValueDataHolderAppendable
    for NexusDataset<D, NexusLogValueResizable>
{
    fn append(
        &self,
        parent: &Self::HDF5Container,
        values: &VectorOfScalars,
    ) -> Result<(), NexusDatasetError> {
        if values.type_descriptor() != self.definition.type_descriptor() {
            return NexusLogValueError::TypeMismatch { required_type: self.definition.type_descriptor(), input_type: values.type_descriptor() }
        }
        let dataset = self.create_hdf5_instance(parent)?;
        let size = dataset.size();
        let next_values_slice = s![size..(size + values.len())];
        dataset.resize(size + values.len())?;
        match values {
            VectorOfScalars::I1(vec) => dataset.write_slice(vec, next_values_slice),
            VectorOfScalars::I2(vec) => dataset.write_slice(vec, next_values_slice),
            VectorOfScalars::I4(vec) => dataset.write_slice(vec, next_values_slice),
            VectorOfScalars::I8(vec) => dataset.write_slice(vec, next_values_slice),
            VectorOfScalars::U1(vec) => dataset.write_slice(vec, next_values_slice),
            VectorOfScalars::U2(vec) => dataset.write_slice(vec, next_values_slice),
            VectorOfScalars::U4(vec) => dataset.write_slice(vec, next_values_slice),
            VectorOfScalars::U8(vec) => dataset.write_slice(vec, next_values_slice),
            VectorOfScalars::F4(vec) => dataset.write_slice(vec, next_values_slice),
            VectorOfScalars::F8(vec) => dataset.write_slice(vec, next_values_slice),
        }?;
        Ok(())
    }
}

impl<D, C, M, R> NexusPushMessage<M, Group, R> for NexusDataset<D, C>
where
    D: NexusDatasetDef + NexusHandleMessage<M, Dataset, R>,
    C: NexusClassDataHolder,
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
    C: NexusClassDataHolder,
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
