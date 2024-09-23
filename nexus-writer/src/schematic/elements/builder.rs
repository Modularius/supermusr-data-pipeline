use hdf5::H5Type;
use std::marker::PhantomData;

use super::{
    NexusBuilderBegun, NexusBuilderFinished, NexusDataHolder, NexusClassDataHolder,
    NexusTypedDataHolder,
};

/// Class of NexusDataHolder which has a mutable scalar value with customizable default
#[derive(Default, Clone)]
pub(crate) struct NexusClassMutableDataHolder<T: H5Type + Default + Clone> {
    pub(super) default_value: T,
}

impl<T: H5Type + Default + Clone> NexusClassDataHolder for NexusClassMutableDataHolder<T> {}

/// Class of NexusDataHolder which has an immutable scalar value with customizable fixed value
#[derive(Default, Clone)]
pub(crate) struct NexusClassFixedDataHolder<T: H5Type + Default + Clone> {
    pub(super) fixed_value: T,
}

impl<T: H5Type + Default + Clone> NexusClassDataHolder for NexusClassFixedDataHolder<T> {}

/// Class of NexusDataHolder whose size can be queried
pub(crate) trait NexusClassDataHolderWithSize : NexusClassDataHolder {}

/// Class of NexusDataHolder which has an expandable vector value with customizable default value
#[derive(Default, Clone)]
pub(crate) struct NexusClassAppendableDataHolder<T: H5Type + Default + Clone> {
    pub(super) default_value: T,
    pub(super) default_size: usize,
    pub(super) chunk_size: usize,
}

impl<T: H5Type + Default + Clone> NexusClassDataHolder for NexusClassAppendableDataHolder<T> {}

impl<T: H5Type + Default + Clone> NexusClassDataHolderWithSize for NexusClassAppendableDataHolder<T> {}

/// Class of NexusDataHolder for value log which has an expandable vector value with type defined at runtime
#[derive(Clone)]
pub(crate) struct NexusLogValueResizable {
    pub(super) chunk_size: usize,
}

impl Default for NexusLogValueResizable {
    fn default() -> Self {
        Self {
            chunk_size: Default::default(),
        }
    }
}

impl NexusClassDataHolder for NexusLogValueResizable {}

impl NexusClassDataHolderWithSize for NexusLogValueResizable {}

/// Builder which constructs NexusDataHolder once the required parameters are given
pub(in crate::schematic) struct NexusBuilder<
    C: NexusClassDataHolder,
    H: NexusDataHolder,
    const FINISHED: bool,
> {
    pub(super) name: String,
    pub(super) class: C,
    pub(super) phantom: PhantomData<H>,
}

/// Implementation of unfinished builder with MutableWithDefault class
impl<H> NexusBuilder<NexusClassMutableDataHolder<H::DataType>, H, false>
where
    H: NexusTypedDataHolder,
    NexusBuilder<NexusClassMutableDataHolder<H::DataType>, H, true>: NexusBuilderFinished,
    NexusClassMutableDataHolder<<H as NexusTypedDataHolder>::DataType>: NexusClassDataHolder,
{
    pub(crate) fn finish_with_default_value(
        self,
        default_value: H::DataType,
    ) -> <<Self as NexusBuilderBegun>::FinshedBuilder as NexusBuilderFinished>::BuildType {
        NexusBuilder {
            name: self.name,
            class: NexusClassMutableDataHolder { default_value },
            phantom: PhantomData,
        }
        .finish()
    }
    pub(crate) fn finish_with_auto_default(
        self,
    ) -> <<Self as NexusBuilderBegun>::FinshedBuilder as NexusBuilderFinished>::BuildType {
        NexusBuilder {
            name: self.name,
            class: NexusClassMutableDataHolder::default(),
            phantom: PhantomData,
        }
        .finish()
    }
}

/// Implementation of unfinished builder with Constant class
impl<H: NexusTypedDataHolder> NexusBuilder<NexusClassFixedDataHolder<H::DataType>, H, false>
where
    H: NexusDataHolder,
    NexusBuilder<NexusClassFixedDataHolder<H::DataType>, H, true>: NexusBuilderFinished,
{
    pub(crate) fn finish_with_fixed_value(
        self,
        fixed_value: H::DataType,
    ) -> <<Self as NexusBuilderBegun>::FinshedBuilder as NexusBuilderFinished>::BuildType {
        NexusBuilder {
            name: self.name,
            class: NexusClassFixedDataHolder { fixed_value },
            phantom: PhantomData,
        }
        .finish()
    }
}

/// Implementation of unfinished builder with Resizable class
impl<H> NexusBuilder<NexusClassAppendableDataHolder<H::DataType>, H, false>
where
    H: NexusTypedDataHolder,
    NexusBuilder<NexusClassAppendableDataHolder<H::DataType>, H, true>: NexusBuilderFinished,
{
    pub(crate) fn finish_with_resizable(
        self,
        default_value: H::DataType,
        default_size: usize,
        chunk_size: usize,
    ) -> <<Self as NexusBuilderBegun>::FinshedBuilder as NexusBuilderFinished>::BuildType {
        NexusBuilder {
            name: self.name,
            class: NexusClassAppendableDataHolder {
                default_value,
                default_size,
                chunk_size,
            },
            phantom: PhantomData,
        }
        .finish()
    }
}

/// Implementation of unfinished builder with log value resizable class
impl<H> NexusBuilder<NexusLogValueResizable, H, false>
where
    H: NexusDataHolder,
    NexusBuilder<NexusLogValueResizable, H, true>: NexusBuilderFinished,
{
    pub(crate) fn finish_log_value_with_resizable(
        self,
        chunk_size: usize,
    ) -> <<Self as NexusBuilderBegun>::FinshedBuilder as NexusBuilderFinished>::BuildType {
        NexusBuilder {
            name: self.name,
            class: NexusLogValueResizable {
                chunk_size,
            },
            phantom: PhantomData,
        }
        .finish()
    }
}

/// Implementation of NexusBuilderBegun trait for unfinished builder
impl<C, H> NexusBuilderBegun for NexusBuilder<C, H, false>
where
    NexusBuilder<C, H, true>: NexusBuilderFinished,
    C: NexusClassDataHolder,
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
