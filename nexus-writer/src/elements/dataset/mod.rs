use hdf5::{Dataset, Group};

use crate::{
    error::{HDF5Error, NexusDatasetError, NexusPushError},
    schematic::H5String,
};

use super::{
    dataholder_class::{
        NexusClassAppendableDataHolder, NexusClassDataHolder, NexusClassFixedDataHolder,
        NexusClassMutableAppendableDataHolder, NexusClassMutableDataHolder,
        NexusClassNumericAppendableDataHolder, NexusClassWithSize, NexusClassWithStaticDataType,
    },
    traits::{
        NexusDataHolder, NexusDataHolderWithSize, NexusDataHolderWithStaticType, NexusDatasetDef,
        NexusH5CreatableDataHolder, NexusH5InstanceCreatableDataHolder, NexusHandleMessage,
        NexusPushMessage,
    },
};

mod dataset;
mod vector;

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
    Self: NexusH5InstanceCreatableDataHolder
        + NexusDataHolder<HDF5Type = Dataset, ThisError = NexusDatasetError>,
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

/*
NexusPushMessage
    */

impl<D, C, M, R> NexusPushMessage<M, Group, R> for NexusDataset<D, C>
where
    D: NexusDatasetDef + NexusHandleMessage<M, Dataset, R>,
    C: NexusClassDataHolder,
    Self: NexusH5InstanceCreatableDataHolder
        + NexusDataHolder<HDF5Container = Group, HDF5Type = Dataset>,
    NexusPushError: From<<Self as NexusDataHolder>::ThisError>,
{
    fn push_message(&mut self, message: &M, parent: &Group) -> Result<R, NexusPushError> {
        let dataset = self.create_hdf5_instance(parent)?;
        let ret = self.definition.handle_message(message, &dataset)?;
        Ok(ret)
    }
}

/*
Conditional Generic Traits
    */
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

impl<D, C> NexusDataHolderWithStaticType for NexusDataset<D, C>
where
    D: NexusDatasetDef,
    C: NexusClassWithStaticDataType,
{
    type DataType = C::DataType;
}
