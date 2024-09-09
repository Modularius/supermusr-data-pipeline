use crate::schematic::elements::{
    error::{ClosingError, CreationError, HDF5Error, OpeningError},
    traits::{self, Class},
    NxLivesInGroup,
};
use hdf5::{Dataset, Group, H5Type};
use tracing::instrument;

use super::{AttributeRegister, NxDataset};

// Implement Database Classes

#[derive(Default)]
pub(crate) struct UnderlyingNexusDataset<
    T,
    D: NxDataset = (),
    C: traits::tags::Tag<T, Group, Dataset> = traits::tags::Mutable,
> where
    T: H5Type + Clone,
{
    pub(super) name: String,
    pub(super) attributes_register: AttributeRegister,
    pub(super) attributes: D,
    pub(super) class: C::ClassType,
    pub(super) dataset: Option<Dataset>,
}

impl<T, D, C> NxLivesInGroup for UnderlyingNexusDataset<T, D, C>
where
    T: H5Type + Clone,
    D: NxDataset,
    C: traits::tags::Tag<T, Group, Dataset>,
{
    #[instrument(skip_all, level = "debug", fields(name = tracing::field::Empty), err(level = "error"))]
    fn create(&mut self, parent: &Group) -> Result<(), CreationError> {
        if self.dataset.is_some() {
            Err(CreationError::AlreadyOpen)
        } else {
            let dataset = self.class.create(parent, &self.name)?;
            for attribute in self.attributes_register.lock_mutex().iter_mut() {
                attribute.lock().expect("Lock Exists").create(&dataset)?;
            }
            self.dataset = Some(dataset);
            Ok(())
        }
    }

    #[instrument(skip_all, level = "debug", fields(name = tracing::field::Empty), err(level = "error"))]
    fn open(&mut self, parent: &Group) -> Result<(), OpeningError> {
        if self.dataset.is_some() {
            Err(OpeningError::AlreadyOpen)
        } else {
            let dataset = parent.dataset(&self.name).map_err(HDF5Error::General)?;
            for attribute in self.attributes_register.lock_mutex().iter_mut() {
                attribute.lock().expect("Lock Exists").open(&dataset)?;
            }
            self.dataset = Some(dataset);
            Ok(())
        }
    }

    #[instrument(skip_all, level = "debug", fields(name = tracing::field::Empty), err(level = "error"))]
    fn close(&mut self) -> Result<(), ClosingError> {
        if self.dataset.is_none() {
            Err(ClosingError::AlreadyClosed)
        } else {
            for attribute in self.attributes_register.lock_mutex().iter_mut() {
                attribute.lock().expect("Lock Exists").close()?;
            }
            self.dataset = None;
            Ok(())
        }
    }
}
