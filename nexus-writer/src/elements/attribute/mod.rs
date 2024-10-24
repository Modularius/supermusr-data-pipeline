use hdf5::{Attribute, Dataset, Group, H5Type};
use std::marker::PhantomData;

use crate::{
    elements::{
        dataholder_class::{
            NexusClassDataHolder, NexusClassFixedDataHolder, NexusClassMutableDataHolder,
            NexusClassWithStaticDataType,
        },
        traits::{
            NexusContainerWithAttribute, NexusDataHolder, NexusDataHolderWithStaticType,
            NexusH5CreatableDataHolder, NexusH5InstanceCreatableDataHolder,
        },
    },
    error::{HDF5Error, NexusAttributeError}
};

mod attribute;

#[derive(Clone)]
pub(crate) struct NexusAttribute<C: NexusClassDataHolder, P: NexusContainerWithAttribute> {
    name: String,
    class: C,
    attribute: Option<Attribute>,
    phantom: PhantomData<P>,
}

pub(crate) type NexusAttributeMut<T, P = Dataset> =
    NexusAttribute<NexusClassMutableDataHolder<T>, P>;

pub(crate) type NexusAttributeFixed<T, P = Dataset> =
    NexusAttribute<NexusClassFixedDataHolder<T>, P>;

impl NexusContainerWithAttribute for Dataset {
    fn attribute<T: H5Type>(&self, name: &str) -> Result<Attribute, NexusAttributeError> {
        self.attr(name)
            .or_else(|_| Ok(self.new_attr::<T>().create(name).map_err(HDF5Error::HDF5)?))
    }
}

impl NexusContainerWithAttribute for Group {
    fn attribute<T: H5Type>(&self, name: &str) -> Result<Attribute, NexusAttributeError> {
        self.attr(name)
            .or_else(|_| Ok(self.new_attr::<T>().create(name).map_err(HDF5Error::HDF5)?))
    }
}

/*
Generic Traits
    */
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
    Self: NexusH5InstanceCreatableDataHolder
        + NexusDataHolder<HDF5Type = Attribute, ThisError = NexusAttributeError>,
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
Conditional Generic Traits
    */

impl<C, P> NexusDataHolderWithStaticType for NexusAttribute<C, P>
where
    C: NexusClassWithStaticDataType,
    P: NexusContainerWithAttribute,
{
    type DataType = C::DataType;
}