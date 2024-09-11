use hdf5::{Dataset, Group, H5Type, Location, SimpleExtents};
use ndarray::s;
use std::marker::PhantomData;

use super::{
    attribute::NexusAttribute, builder::{
        NexusBuilder, NexusDataHolderConstant, NexusDataHolderMutable, NexusDataHolderResizable,
    }, NexusBuildable, NexusBuilderBegun, NexusBuilderFinished, NexusDataHolder, NexusDataHolderAppendable, NexusDataHolderClass, NexusDataHolderScalarMutable, NexusDatasetDef, NexusError, NexusPushMessage, NexusPushMessageMut
};

impl<T, C, D> NexusBuilderFinished for NexusBuilder<C, NexusDataset<T, D, C>, true>
where
    T: H5Type + Clone + Default,
    C: NexusDataHolderClass,
    D: NexusDatasetDef,
    NexusDataset<T, D, C>: NexusDataHolder,
{
    type BuildType = NexusDataset<T, D, C>;

    fn finish(self) -> NexusDataset<T, D, C> {
        NexusDataset {
            name: self.name,
            class: self.class,
            dataset: None,
            definition: D::new(),
            phantom: PhantomData,
        }
    }
}

pub(in crate::schematic) struct NexusDataset<
    T: H5Type + Clone + Default,
    D: NexusDatasetDef = (),
    C: NexusDataHolderClass = NexusDataHolderMutable<T>,
> {
    name: String,
    class: C,
    dataset: Option<Dataset>,
    definition: D,
    phantom: PhantomData<T>,
}

impl<T,D,C> NexusDataset<T,D,C> where
T: H5Type + Clone + Default,
D: NexusDatasetDef,
C: NexusDataHolderClass {
    pub(crate) fn attribute<F, T2, C2>(&self, f : F) -> &NexusAttribute<T2, C2>
    where 
    F : Fn(&D) -> &NexusAttribute<T2, C2>,
    T2: H5Type + Clone + Default,
    C2: NexusDataHolderClass
    {
        f(&self.definition)
    }
}

pub(in crate::schematic) type NexusDatasetFixed<T, D = ()> = NexusDataset<T, D, NexusDataHolderConstant<T>>;

pub(in crate::schematic) type NexusDatasetResize<T, D = ()> = NexusDataset<T, D, NexusDataHolderResizable<T>>;

impl<T, D, C> NexusBuildable for NexusDataset<T, D, C>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
    C: NexusDataHolderClass,
    NexusDataset<T, D, C>: NexusDataHolder,
{
    type Builder = NexusBuilder<C, NexusDataset<T, D, C>, false>;
    
    fn begin(name: &str) -> Self::Builder {
        Self::Builder::new(name)
    }

}

impl<T, D> NexusDataHolder for NexusDataset<T, D, NexusDataHolderMutable<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
    NexusDataHolderMutable<T>: NexusDataHolderClass,
{
    type DataType = T;
    type HDF5Type = Dataset;
    type HDF5Container = Group;

    fn create_hdf5(&self, parent: &Self::HDF5Container) -> Result<(), NexusError> {
        let dataset = parent.dataset(&self.name).or_else(|_| {
            let dataset = parent
                .new_dataset::<T>()
                .create(self.name.as_str())
                .map_err(|_| NexusError::Unknown)?;
            dataset
                .write_scalar(&self.class.default_value)
                .map_err(|_| NexusError::Unknown)?;
            Ok(dataset)
        })?;
        //self.dataset = Some(dataset);
        Ok(())
    }

    fn close_hdf5(&mut self) {
        self.dataset = None;
    }
}

impl<T, D> NexusDataHolder for NexusDataset<T, D, NexusDataHolderConstant<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
{
    type DataType = T;
    type HDF5Type = Dataset;
    type HDF5Container = Group;

    fn create_hdf5(&self, parent: &Self::HDF5Container) -> Result<(), NexusError> {
        let dataset = parent.dataset(&self.name).or_else(|_| {
            let dataset = parent
                .new_dataset::<T>()
                .create(self.name.as_str())
                .map_err(|_| NexusError::Unknown)?;
            dataset
                .write_scalar(&self.class.fixed_value)
                .map_err(|_| NexusError::Unknown)?;
            Ok(dataset)
        })?;
        //self.dataset = Some(dataset);
        Ok(())
    }

    fn close_hdf5(&mut self) {
        self.dataset = None;
    }
}

impl<T, D> NexusDataHolder for NexusDataset<T, D, NexusDataHolderResizable<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
{
    type DataType = T;
    type HDF5Type = Dataset;
    type HDF5Container = Group;

    fn create_hdf5(&self, parent: &Self::HDF5Container) -> Result<(), NexusError> {
        let dataset = parent
            .new_dataset::<T>()
            .shape(SimpleExtents::resizable(vec![self.class.default_size]))
            .chunk(vec![self.class.chunk_size])
            .create(self.name.as_str())
            .map_err(|_| NexusError::Unknown)?;
        dataset
            .write_slice(
                &vec![self.class.default_value.clone(); self.class.default_size],
                s![0..self.class.default_size],
            )
            .map_err(|_| NexusError::Unknown)?;
        //self.dataset = Some(dataset);
        Ok(())
    }

    fn close_hdf5(&mut self) {
        self.dataset = None;
    }
}

impl<T, D> NexusDataHolderScalarMutable for NexusDataset<T, D, NexusDataHolderMutable<T>>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef,
    NexusDataHolderMutable<T>: NexusDataHolderClass,
{
    fn write_scalar(&self, value: Self::DataType) -> Result<(), NexusError> {
        self.dataset.as_ref().ok_or(NexusError::Unknown).and_then(|dataset| {
            dataset
                .write_scalar(&value)
                .map_err(|_| NexusError::Unknown)
        })
    }

    fn read_scalar(&self) -> Result<Self::DataType, NexusError> {
        self.dataset.as_ref()
            .ok_or(NexusError::Unknown)
            .and_then(|dataset| dataset.read_scalar().map_err(|_| NexusError::Unknown))
    }
}

impl<D: NexusDatasetDef, T: H5Type + Clone + Default> NexusDataHolderAppendable
    for NexusDataset<T, D, NexusDataHolderResizable<T>>
{
    fn append(&self, values: &[Self::DataType]) -> Result<(), NexusError> {
        self.dataset
            .as_ref()
            .ok_or_else(|| NexusError::Unknown)
            .and_then(|dataset| {
                let size = dataset.size();
                let next_values_slice = s![size..(size + values.len())];
                dataset
                    .resize(size + values.len())
                    .map_err(|_| NexusError::Unknown)?;
                dataset
                    .write_slice(values, next_values_slice)
                    .map_err(|_| NexusError::Unknown)
            })
    }

    fn get_size(&self) -> Result<usize, NexusError> {
        self.dataset.as_ref()
            .ok_or(NexusError::Unknown)
            .map(|dataset| dataset.size())
    }
}

impl<T, D, C, M> NexusPushMessage<Dataset, M> for NexusDataset<T, D, C>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef + NexusPushMessage<Dataset, M>,
    C: NexusDataHolderClass,
{
    fn push_message(&self, message: &M, dataset: &Dataset) -> Result<(), NexusError> {
        self.definition.push_message(message, dataset)
    }
}

impl<T, D, C, M> NexusPushMessageMut<Group, M> for NexusDataset<T, D, C>
where
    T: H5Type + Clone + Default,
    D: NexusDatasetDef + NexusPushMessageMut<Group, M>,
    C: NexusDataHolderClass,
{
    fn push_message_mut(&mut self, message: &M, group: &Group) -> Result<(), NexusError> {
        self.definition.push_message_mut(message, group)
    }
}
