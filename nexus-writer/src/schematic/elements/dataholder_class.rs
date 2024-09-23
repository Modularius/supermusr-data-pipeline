use hdf5::{types::TypeDescriptor, H5Type};

use crate::error::NexusNumericError;

/// Implemented for objects in `builder.rs` which serve as classes for `NexusDataHolder` objects
/// i.e. `NexusDataMutable`, `NexusDataHolderConstant` and `NexusDataHolderResizable`
pub(crate) trait NexusClassDataHolder: Default + Clone {}

/// Class of NexusDataHolder which has a mutable scalar value with customizable default
#[derive(Default, Clone)]
pub(crate) struct NexusClassMutableDataHolder<T: H5Type + Default + Clone> {
    pub(super) default_value: T,
}

impl<T: H5Type + Default + Clone> NexusClassDataHolder for NexusClassMutableDataHolder<T> {}

impl<T: H5Type + Default + Clone> NexusClassWithStaticDataType<T>
    for NexusClassMutableDataHolder<T>
{
}

/// Class of NexusDataHolder which has an immutable scalar value with customizable fixed value
#[derive(Default, Clone)]
pub(crate) struct NexusClassFixedDataHolder<T: H5Type + Default + Clone> {
    pub(super) fixed_value: T,
}

impl<T: H5Type + Default + Clone> NexusClassDataHolder for NexusClassFixedDataHolder<T> {}

impl<T: H5Type + Default + Clone> NexusClassWithStaticDataType<T> for NexusClassFixedDataHolder<T> {}

/// Class of NexusDataHolder whose size can be queried
pub(crate) trait NexusClassWithSize: NexusClassDataHolder {}

/// Class of NexusDataHolder whose size can be queried
pub(crate) trait NexusClassWithStaticDataType<T: H5Type + Clone + Default>:
    NexusClassDataHolder
{
}

/// Class of NexusDataHolder which has an expandable vector value with customizable default value
#[derive(Default, Clone)]
pub(crate) struct NexusClassAppendableDataHolder<T: H5Type + Default + Clone> {
    pub(super) default_value: T,
    pub(super) default_size: usize,
    pub(super) chunk_size: usize,
}

impl<T: H5Type + Default + Clone> NexusClassDataHolder for NexusClassAppendableDataHolder<T> {}

impl<T: H5Type + Default + Clone> NexusClassWithStaticDataType<T>
    for NexusClassAppendableDataHolder<T>
{
}

impl<T: H5Type + Default + Clone> NexusClassWithSize for NexusClassAppendableDataHolder<T> {}

/// Class of NexusDataHolder for value log which has an expandable vector value with type defined at runtime
#[derive(Default, Clone)]
pub(crate) struct NexusClassNumericAppendableDataHolder {
    pub(super) type_desc: Option<TypeDescriptor>,
    pub(super) chunk_size: usize,
}

impl NexusClassNumericAppendableDataHolder {
    pub(crate) fn try_set_type(
        &mut self,
        init_type_desc: TypeDescriptor,
    ) -> Result<(), NexusNumericError> {
        if let Some(type_desc) = &self.type_desc {
            if *type_desc != init_type_desc {
                Err(NexusNumericError::TypeMismatch {
                    required_type: type_desc.clone(),
                    input_type: init_type_desc,
                })?;
            }
        } else {
            self.type_desc = Some(init_type_desc);
        }
        Ok(())
    }
}

impl NexusClassDataHolder for NexusClassNumericAppendableDataHolder {}

impl NexusClassWithSize for NexusClassNumericAppendableDataHolder {}
