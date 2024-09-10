use std::marker::PhantomData;

use hdf5::{Attribute, Dataset, Group, H5Type};

use super::{
    builder::{NexusBuilder, NexusDataHolderConstant, NexusDataHolderMutable}, NexusBuildable, NexusBuilderFinished, NexusDataHolder, NexusDataHolderClass, NexusDataHolderScalarMutable, NexusError
};

impl<T: H5Type + Clone + Default, C: NexusDataHolderClass> NexusBuilderFinished<NexusAttribute<T, C>>
    for NexusBuilder<C, NexusAttribute<T, C>, true>
{
    fn finish(self) -> Result<NexusAttribute<T, C>, super::NexusError> {
        Ok(NexusAttribute {
            name: self.name,
            class: self.class,
            attribute: None,
            phantom: PhantomData
        })
    }
}

pub(crate) struct NexusAttribute<T: H5Type + Clone + Default, C: NexusDataHolderClass = NexusDataHolderMutable<T>> {
    name: String,
    class: C,
    attribute: Option<Attribute>,
    phantom: PhantomData<T>
}
pub(crate) type NexusAttributeFixed<T> = NexusAttribute<T, NexusDataHolderConstant<T>>;

impl<T: H5Type + Clone + Default, C: NexusDataHolderClass> NexusBuildable for NexusAttribute<T, C> {
    type Builder = NexusBuilder<C, NexusAttribute<T, C>, false>;

    fn begin(name: &str) -> Self::Builder {
        NexusBuilder {
            name: name.to_string(),
            class: C::default(),
            phantom: PhantomData,
        }
    }
}

impl<T: H5Type + Clone + Default> NexusDataHolder
    for NexusAttribute<T, NexusDataHolderMutable<T>>
{
    type DataType = T;
    type HDF5Type = Attribute;
    type HDF5Container = Dataset;

    fn create_hdf5(
        &mut self,
        parent: &Self::HDF5Container,
    ) -> Result<Self::HDF5Type, NexusError> {
        self.dataset = Some(
            parent.dataset(&self.name).or_else(||{
                let dataset = parent
                    .new_dataset::<T>()
                    .create(self.name)
                    .map_err(|_|NexusError)?;
                dataset.write_scalar(&self.class.default_value).map_err(|_|NexusError)?
            })
        );
    }
    fn close_hdf5(&mut self) {
        self.dataset = None;
    }
}

impl<T: H5Type + Clone + Default> NexusDataHolderScalarMutable
    for NexusAttribute<T, NexusDataHolderMutable<T>> {
    fn write_scalar(&self, value: Self::DataType) -> Result<(), NexusError> {
        if let Some(dataset) = self.dataset {
            dataset.write_scalar(&value)?;
        }
        Ok(())
    }
    
    fn read_scalar(&self) -> Result<Self::DataType, NexusError> {
        self.dataset.ok_or(NexusError).map(|dataset|dataset.read_scalar()?)
    }
}