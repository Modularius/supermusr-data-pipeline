use std::marker::PhantomData;

use hdf5::H5Type;

use super::{NexusBuilderBegun, NexusBuilderFinished, NexusDataHolder, NexusDataHolderClass};

/// Class of NexusDataHolder which has a mutable scalar value with customizable default
#[derive(Default, Clone)]
pub(crate) struct NexusDataHolderMutable<T: H5Type + Default + Clone> {
    pub(super) default_value: T,
}

impl<T: H5Type + Default + Clone> NexusDataHolderClass for NexusDataHolderMutable<T> {}

/// Class of NexusDataHolder which has an immutable scalar value with customizable fixed value
#[derive(Default, Clone)]
pub(crate) struct NexusDataHolderConstant<T: H5Type + Default + Clone> {
    pub(super) fixed_value: T,
}

impl<T: H5Type + Default + Clone> NexusDataHolderClass for NexusDataHolderConstant<T> {}

/// Class of NexusDataHolder which has an expandable vector value with customizable default value
#[derive(Default, Clone)]
pub(crate) struct NexusDataHolderResizable<T: H5Type + Default + Clone> {
    pub(super) default_value: T,
    pub(super) default_size: usize,
    pub(super) chunk_size: usize,
}

impl<T: H5Type + Default + Clone> NexusDataHolderClass for NexusDataHolderResizable<T> {}

/// Builder which constructs NexusDataHolder once the required parameters are given
pub(in crate::schematic) struct NexusBuilder<
    C: NexusDataHolderClass,
    H: NexusDataHolder,
    const FINISHED: bool,
> {
    pub(super) name: String,
    pub(super) class: C,
    pub(super) phantom: PhantomData<H>,
}

/// Implementation of unfinished builder with MutableWithDefault class
impl<H> NexusBuilder<NexusDataHolderMutable<H::DataType>, H, false>
where
    H: NexusDataHolder,
    NexusBuilder<NexusDataHolderMutable<H::DataType>, H, true>: NexusBuilderFinished,
    NexusDataHolderMutable<<H as NexusDataHolder>::DataType>: NexusDataHolderClass,
{
    pub(crate) fn default_value(
        self,
        default_value: H::DataType,
    ) -> <Self as NexusBuilderBegun>::FinshedBuilder {
        NexusBuilder {
            name: self.name,
            class: NexusDataHolderMutable { default_value },
            phantom: PhantomData,
        }
    }
    pub(crate) fn auto_default(
        self
    ) -> <Self as NexusBuilderBegun>::FinshedBuilder {
        NexusBuilder {
            name: self.name,
            class: NexusDataHolderMutable::default(),
            phantom: PhantomData,
        }
    }
}

/// Implementation of unfinished builder with Constant class
impl<H: NexusDataHolder> NexusBuilder<NexusDataHolderConstant<H::DataType>, H, false>
where
    H: NexusDataHolder,
    NexusBuilder<NexusDataHolderConstant<H::DataType>, H, true>: NexusBuilderFinished,
{
    pub(crate) fn fixed_value(
        self,
        fixed_value: H::DataType,
    ) -> <Self as NexusBuilderBegun>::FinshedBuilder {
        NexusBuilder {
            name: self.name,
            class: NexusDataHolderConstant { fixed_value },
            phantom: PhantomData,
        }
    }
}

/// Implementation of unfinished builder with Resizable class
impl<H> NexusBuilder<NexusDataHolderResizable<H::DataType>, H, false>
where
    H: NexusDataHolder,
    NexusBuilder<NexusDataHolderResizable<H::DataType>, H, true>: NexusBuilderFinished,
{
    pub(crate) fn resizable(
        self,
        default_value: H::DataType,
        default_size: usize,
        chunk_size: usize,
    ) -> <Self as NexusBuilderBegun>::FinshedBuilder {
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

/// Implementation of NexusBuilderBegun trait for unfinished builder
impl<C, H> NexusBuilderBegun for NexusBuilder<C, H, false>
where
    NexusBuilder<C, H, true>: NexusBuilderFinished,
    C: NexusDataHolderClass,
    H: NexusDataHolder,
{
    type FinshedBuilder = NexusBuilder<C, H, true>;

    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            class: C::default(),
            phantom: PhantomData,
        }
    }
}
