use std::marker::PhantomData;

use hdf5::H5Type;

use super::{NexusBuilderBegun, NexusDataHolder, NexusDataHolderClass};

#[derive(Default, Clone)]
pub(crate) struct NexusDataHolderMutable<T: H5Type + Default + Clone> {
    pub(super) default_value: T,
}

impl<T: H5Type + Default + Clone> NexusDataHolderClass for NexusDataHolderMutable<T> {}

#[derive(Default, Clone)]
pub(crate) struct NexusDataHolderConstant<T: H5Type + Default + Clone> {
    pub(super) fixed_value: T,
}

impl<T: H5Type + Default + Clone> NexusDataHolderClass for NexusDataHolderConstant<T> {}

#[derive(Default, Clone)]
pub(crate) struct NexusDataHolderResizable<T: H5Type + Default + Clone> {
    pub(super) default_value: T,
    pub(super) default_size: usize,
    pub(super) chunk_size: usize,
}

impl<T: H5Type + Default + Clone> NexusDataHolderClass for NexusDataHolderResizable<T> {}

pub(super) struct NexusBuilder<C: NexusDataHolderClass, H: NexusDataHolder, const FINISHED: bool> {
    pub(super) name: String,
    pub(super) class: C,
    pub(super) phantom: PhantomData<H>,
}

impl<H: NexusDataHolder> NexusBuilder<NexusDataHolderMutable<H::DataType>, H, false> {
    pub(crate) fn default_value(
        self,
        default_value: H::DataType,
    ) -> <Self as NexusBuilderBegun<H>>::FinshedBuilder {
        NexusBuilder {
            name: self.name,
            class: NexusDataHolderMutable { default_value },
            phantom: PhantomData,
        }
    }
}

impl<D, H: NexusDataHolder> NexusBuilder<NexusDataHolderConstant<H::DataType>, H, false> {
    pub(crate) fn fixed_value(
        self,
        fixed_value: H::DataType,
    ) -> <Self as NexusBuilderBegun<H>>::FinshedBuilder {
        NexusBuilder {
            name: self.name,
            class: NexusDataHolderConstant { fixed_value },
            phantom: PhantomData,
        }
    }
}

impl<H: NexusDataHolder> NexusBuilder<NexusDataHolderResizable<H::DataType>, H, false> {
    pub(crate) fn resizable(
        self,
        default_value: H::DataType,
        default_size: usize,
        chunk_size: usize,
    ) -> <Self as NexusBuilderBegun<H>>::FinshedBuilder {
        NexusBuilder {
            name: self.name,
            class: NexusDataHolderResizable {
                default_value,
                default_size,
                chunk_size,
            },
            phantom: PhantomData,
        }
    }
}

impl<C: NexusDataHolderClass, H: NexusDataHolder> NexusBuilderBegun<H>
    for NexusBuilder<C, H, false>
{
    type FinshedBuilder = NexusBuilder<C, H, true>;
}
