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

mod scalar;

#[derive(Clone)]
pub(crate) struct NexusAttribute<C: NexusClassDataHolder, P: NexusContainerWithAttribute> {
    name: String,
    class: C,
    phantom: PhantomData<P>,
}

pub(crate) type NexusAttributeMut<T, P = Dataset> =
    NexusAttribute<NexusClassMutableDataHolder<T>, P>;

pub(crate) type NexusAttributeFixed<T, P = Dataset> =
    NexusAttribute<NexusClassFixedDataHolder<T>, P>;

impl NexusContainerWithAttribute for Dataset {
    fn attribute<T, F>(&self, name: &str, f : F) -> Result<Attribute, NexusAttributeError> where
    T: H5Type, F : Fn(Attribute)->Result<Attribute,HDF5Error> {
        self.attr(name)
            .or_else(|_| Ok(self.new_attr::<T>().create(name).map_err(HDF5Error::HDF5).and_then(f)?))
    }
}

impl NexusContainerWithAttribute for Group {
    fn attribute<T, F>(&self, name: &str, f : F) -> Result<Attribute, NexusAttributeError> where
    T: H5Type, F : Fn(Attribute)->Result<Attribute,HDF5Error> {
        self.attr(name)
            .or_else(|_| Ok(self.new_attr::<T>().create(name).map_err(HDF5Error::HDF5).and_then(f)?))
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