use hdf5::{types::TypeDescriptor, Dataset, Group, H5Type, SimpleExtents};
use ndarray::s;

use crate::{
    error::{HDF5Error, NexusDatasetError, NexusNumericError, NexusPushError},
    schematic::H5String,
};

use super::{
    dataholder_class::{
        NexusClassAppendableDataHolder, NexusClassDataHolder, NexusClassFixedDataHolder,
        NexusClassMutableAppendableDataHolder, NexusClassMutableDataHolder,
        NexusClassNumericAppendableDataHolder, NexusClassWithSize,
    },
    log_value::{DatasetBuilderNumericExt, DatasetNumericExt, NumericVector},
    traits::{
        NexusAppendableDataHolder, NexusDataHolder, NexusDataHolderFixed,
        NexusDataHolderScalarMutable, NexusDataHolderStringMutable, NexusDataHolderVectorMutable,
        NexusDataHolderWithSize, NexusDataHolderWithStaticType, NexusDatasetDef,
        NexusH5CreatableDataHolder, NexusH5InstanceCreatableDataHolder, NexusHandleMessage,
        NexusNumericAppendableDataHolder, NexusPushMessage, NexusSearchableDataHolder,
    },
};

#[derive(Clone, Default)]
pub(crate) struct NexusDataset<D: NexusDatasetDef, C: NexusClassDataHolder> {
    name: String,
    class: C,
    dataset: Option<Dataset>,
    definition: D,
}

pub(crate) type NexusDatasetMut<T, D = ()> = NexusDataset<D, NexusClassMutableDataHolder<T>>;

pub(crate) type NexusDatasetFixed<T, D = ()> = NexusDataset<D, NexusClassFixedDataHolder<T>>;

pub(crate) type NexusDatasetResize<T, D = ()> = NexusDataset<D, NexusClassAppendableDataHolder<T>>;

pub(crate) type NexusDatasetResizeMut<T, D = ()> =
    NexusDataset<D, NexusClassMutableAppendableDataHolder<T>>;

pub(crate) type NexusLogValueDatasetResize<D = ()> =
    NexusDataset<D, NexusClassNumericAppendableDataHolder>;

impl<D, C> NexusDataset<D, C>
where
    D: NexusDatasetDef,
    C: NexusClassDataHolder,
{
    fn create_units(&self, dataset: &Dataset) -> Result<(), HDF5Error> {
        if let Some(units) = D::UNITS {
            let attribute = dataset.new_attr::<H5String>().create("units")?;
            attribute.write_scalar(&units.to_string().parse::<H5String>().expect(""))?;
        }
        Ok(())
    }
}

/*
Generic Traits
    */
impl<D, C> NexusDataHolder for NexusDataset<D, C>
where
    D: NexusDatasetDef,
    C: NexusClassDataHolder,
{
    type HDF5Type = Dataset;
    type HDF5Container = Group;
    type ThisError = NexusDatasetError;
}

impl<D, C> NexusH5CreatableDataHolder for NexusDataset<D, C>
where
    D: NexusDatasetDef,
    C: NexusClassDataHolder,
    NexusDataset<D, C>:
        NexusH5InstanceCreatableDataHolder<HDF5Type = Dataset, ThisError = NexusDatasetError>,
{
    fn create_hdf5(&mut self, parent: &Self::HDF5Container) -> Result<(), NexusDatasetError> {
        let dataset = self.create_hdf5_instance(parent)?;
        self.dataset = Some(dataset.clone());
        Ok(())
    }
    fn close_hdf5(&mut self) {
        self.dataset = None;
    }
}

impl<D, C> NexusDataHolderWithSize for NexusDataset<D, C>
where
    D: NexusDatasetDef,
    C: NexusClassWithSize,
    NexusDataset<D, C>: NexusH5InstanceCreatableDataHolder<HDF5Type = Dataset>,
{
    fn get_size(&self, parent: &Self::HDF5Container) -> Result<usize, Self::ThisError> {
        let dataset = self.create_hdf5_instance(parent)?;
        Ok(dataset.size())
    }
}

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

impl<T, D> NexusDataHolderWithStaticType for NexusDataset<D, NexusClassMutableDataHolder<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
{
    type DataType = T;
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
            dataset: None,
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

impl<D> NexusDataHolderStringMutable for NexusDataset<D, NexusClassMutableDataHolder<H5String>>
where
    D: NexusDatasetDef,
    Self::ThisError: From<HDF5Error>,
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
        if let Some(ref dataset) = self.dataset {
            Ok(dataset.clone())
        } else {
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
            dataset: None,
            definition: D::new(),
        }
    }

    fn write(&self, parent: &Self::HDF5Container) -> Result<(), Self::ThisError> {
        let dataset = self.create_hdf5_instance(parent)?;
        self.create_units(&dataset)?;
        Ok(())
    }
}

impl<T, D> NexusDataHolderWithStaticType for NexusDataset<D, NexusClassFixedDataHolder<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
{
    type DataType = T;
}

/*
NexusClassAppendableDataHolder
    */

impl<T, D> NexusH5InstanceCreatableDataHolder for NexusDataset<D, NexusClassAppendableDataHolder<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
{
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
                    .create(self.name.as_str())
                    .map_err(HDF5Error::HDF5)?;
                dataset
                    .write_slice(
                        &vec![self.class.default_value.clone(); self.class.default_size],
                        s![0..self.class.default_size],
                    )
                    .map_err(HDF5Error::HDF5)?;
                self.create_units(&dataset)?;
                Ok::<_, NexusDatasetError>(dataset)
            })
        }
    }
}

impl<T, D> NexusDataHolderWithStaticType for NexusDataset<D, NexusClassAppendableDataHolder<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
{
    type DataType = T;
}

impl<D: NexusDatasetDef, T: H5Type + Clone + Default> NexusAppendableDataHolder
    for NexusDataset<D, NexusClassAppendableDataHolder<T>>
{
    fn new_with_initial_size(
        name: &str,
        default_value: Self::DataType,
        default_size: usize,
        chunk_size: usize,
    ) -> Self {
        Self {
            name: name.to_string(),
            class: NexusClassAppendableDataHolder {
                default_value,
                default_size,
                chunk_size,
            },
            dataset: None,
            definition: D::new(),
        }
    }

    fn append(
        &self,
        parent: &Self::HDF5Container,
        values: &[Self::DataType],
    ) -> Result<(), NexusDatasetError> {
        let dataset = self.create_hdf5_instance(parent)?;
        let size = dataset.size();
        let next_values_slice = s![size..(size + values.len())];
        dataset
            .resize(size + values.len())
            .map_err(HDF5Error::HDF5)?;
        Ok(dataset
            .write_slice(values, next_values_slice)
            .map_err(HDF5Error::HDF5)?)
    }
}

/*
NexusClassMutableAppendableDataHolder
    */

impl<T, D> NexusH5InstanceCreatableDataHolder
    for NexusDataset<D, NexusClassMutableAppendableDataHolder<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
{
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
                    .create(self.name.as_str())
                    .map_err(HDF5Error::HDF5)?;
                dataset
                    .write_slice(
                        &vec![self.class.default_value.clone(); self.class.default_size],
                        s![0..self.class.default_size],
                    )
                    .map_err(HDF5Error::HDF5)?;
                self.create_units(&dataset)?;
                Ok::<_, NexusDatasetError>(dataset)
            })
        }
    }
}

impl<T, D> NexusDataHolderWithStaticType
    for NexusDataset<D, NexusClassMutableAppendableDataHolder<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
{
    type DataType = T;
}

impl<D, T> NexusAppendableDataHolder for NexusDataset<D, NexusClassMutableAppendableDataHolder<T>>
where
    D: NexusDatasetDef,
    T: H5Type + Clone + Default + PartialEq,
{
    fn new_with_initial_size(
        name: &str,
        default_value: Self::DataType,
        default_size: usize,
        chunk_size: usize,
    ) -> Self {
        Self {
            name: name.to_string(),
            class: NexusClassMutableAppendableDataHolder {
                default_value,
                default_size,
                chunk_size,
            },
            dataset: None,
            definition: D::new(),
        }
    }

    fn append(
        &self,
        parent: &Self::HDF5Container,
        values: &[Self::DataType],
    ) -> Result<(), NexusDatasetError> {
        let dataset = self.create_hdf5_instance(parent)?;
        let size = dataset.size();
        let next_values_slice = s![size..(size + values.len())];
        dataset
            .resize(size + values.len())
            .map_err(HDF5Error::HDF5)?;
        Ok(dataset
            .write_slice(values, next_values_slice)
            .map_err(HDF5Error::HDF5)?)
    }
}

impl<D, T> NexusSearchableDataHolder for NexusDataset<D, NexusClassAppendableDataHolder<T>>
where
    D: NexusDatasetDef,
    T: H5Type + Clone + Default + PartialEq,
{
    fn find(
        &self,
        parent: &Self::HDF5Container,
        value: Self::DataType,
    ) -> Result<Option<usize>, Self::ThisError> {
        let dataset = self.create_hdf5_instance(parent)?;
        let size = dataset.size();
        let selection = s![0..size];
        let slice = dataset
            .read_slice::<T, _, ndarray::Dim<[usize; 1]>>(selection)
            .map_err(HDF5Error::HDF5)?;
        Ok((0..size).find(|index| {
            *slice
                .get(*index)
                .expect("Slice should have size at least {index}")
                == value
        }))
    }
}

impl<T, D> NexusDataHolderVectorMutable
    for NexusDataset<D, NexusClassMutableAppendableDataHolder<T>>
where
    T: H5Type + Clone + Default + PartialEq,
    D: NexusDatasetDef,
{
    fn write_at(
        &self,
        parent: &Self::HDF5Container,
        value: Self::DataType,
        index: usize,
    ) -> Result<(), NexusDatasetError> {
        let dataset = self.create_hdf5_instance(parent)?;
        let size = dataset.size();
        if index >= size {
            Err(NexusDatasetError::WriteOutOfBounds(index, size))?;
        }
        let slice = s![index..(index + 1)];
        Ok(dataset
            .write_slice(&[value], slice)
            .map_err(HDF5Error::HDF5)?)
    }

    fn read_from(
        &self,
        parent: &Self::HDF5Container,
        index: usize,
    ) -> Result<Self::DataType, Self::ThisError> {
        let dataset = self.create_hdf5_instance(parent)?;
        let size = dataset.size();
        if index >= size {
            Err(NexusDatasetError::ReadOutOfBounds(index, size))?;
        }
        let selection = s![index..(index + 1)];
        let slice = dataset
            .read_slice_1d::<T, _>(selection)
            .map_err(HDF5Error::HDF5)?;
        let data = slice.first().expect("Slice should not be empty");
        Ok(data.clone())
    }

    fn mutate_all_in_place<F>(
        &self,
        parent: &Self::HDF5Container,
        f: F,
    ) -> Result<(), Self::ThisError>
    where
        F: Fn(&Self::DataType) -> Self::DataType,
    {
        let dataset = self.create_hdf5_instance(parent)?;
        let selection = s![0..dataset.size()];
        let mut slice = dataset.read_slice_1d(selection).map_err(HDF5Error::HDF5)?;
        slice.iter_mut().for_each(|value| *value = f(value));
        Ok(())
    }

    fn mutate_in_place<F>(
        &self,
        parent: &Self::HDF5Container,
        index: usize,
        f: F,
    ) -> Result<(), Self::ThisError>
    where
        F: Fn(&Self::DataType) -> Self::DataType,
    {
        let dataset = self.create_hdf5_instance(parent)?;
        let size = dataset.size();
        if index >= size {
            Err(NexusDatasetError::ReadOutOfBounds(index, size))?;
        }
        let selection = s![index..(index + 1)];
        let mut slice = dataset
            .read_slice_1d::<T, _>(selection)
            .map_err(HDF5Error::HDF5)?;
        let value = slice.first_mut().expect("Slice should not be empty");
        *value = f(value);
        Ok(())
    }
}

/*
NexusNumericAppendableDataHolder
    */

impl<D> NexusH5InstanceCreatableDataHolder
    for NexusDataset<D, NexusClassNumericAppendableDataHolder>
where
    D: NexusDatasetDef,
{
    fn create_hdf5_instance(
        &self,
        parent: &Self::HDF5Container,
    ) -> Result<hdf5::Dataset, NexusDatasetError> {
        if let Some(type_desc) = &self.class.type_desc {
            if let Some(ref dataset) = self.dataset {
                Ok(dataset.clone())
            } else {
                let dataset = parent.dataset(&self.name).or_else(|_| {
                    parent
                        .new_dataset_builder()
                        .chunk(vec![self.class.chunk_size])
                        .create_numeric(self.name.as_str(), type_desc)
                })?;
                self.create_units(&dataset)?;
                Ok(dataset)
            }
        } else {
            Err(NexusNumericError::NumericTypeNotSet)?
        }
    }
}

impl<D: NexusDatasetDef> NexusNumericAppendableDataHolder
    for NexusDataset<D, NexusClassNumericAppendableDataHolder>
{
    fn new(name: &str, chunk_size: usize) -> Self {
        Self {
            name: name.to_string(),
            class: NexusClassNumericAppendableDataHolder {
                type_desc: None,
                chunk_size,
            },
            dataset: None,
            definition: D::new(),
        }
    }

    fn try_set_type(&mut self, init_type_desc: TypeDescriptor) -> Result<(), Self::ThisError> {
        if let Some(type_desc) = &self.class.type_desc {
            if *type_desc != init_type_desc {
                Err(NexusNumericError::TypeMismatch {
                    required_type: type_desc.clone(),
                    input_type: init_type_desc,
                })?;
            }
        } else {
            self.class.type_desc = Some(init_type_desc);
        }
        Ok(())
    }

    /// This function assumes values is of the correct type (i.e. that try_set_type has successfully been called).
    fn append_numerics(
        &self,
        parent: &Self::HDF5Container,
        values: &NumericVector,
    ) -> Result<(), NexusDatasetError> {
        let dataset = self.create_hdf5_instance(parent)?;
        let size = dataset.size();
        dataset
            .resize(size + values.len())
            .map_err(HDF5Error::HDF5)?;
        dataset.write_numeric_slice(values, s![size..(size + values.len())])
    }
}

/*
NexusPushMessage
    */

impl<D, C, M, R> NexusPushMessage<M, Group, R> for NexusDataset<D, C>
where
    D: NexusDatasetDef + NexusHandleMessage<M, Dataset, R>,
    C: NexusClassDataHolder,
    Self: NexusH5InstanceCreatableDataHolder<HDF5Container = Group, HDF5Type = Dataset>,
    NexusPushError: From<<Self as NexusDataHolder>::ThisError>,
{
    fn push_message(&mut self, message: &M, parent: &Group) -> Result<R, NexusPushError> {
        let dataset = self.create_hdf5_instance(parent)?;
        let ret = self.definition.handle_message(message.into(), &dataset)?;
        Ok(ret)
    }
}
