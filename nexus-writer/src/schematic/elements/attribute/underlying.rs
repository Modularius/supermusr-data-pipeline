use hdf5::{Attribute, Dataset, H5Type};
use tracing::instrument;

use super::NxAttribute;
use crate::schematic::elements::{error::{ClosingError, CreationError, HDF5Error, OpeningError}, traits::{self, Class}};

pub(crate) struct UnderlyingNexusAttribute<
    T: H5Type,
    C: traits::tags::Tag<T, Dataset, Attribute> = (),
> {
    pub(super) name: String,
    pub(super) class: C::ClassType,
    pub(super) attribute: Option<Attribute>,
}

impl<T: H5Type + Clone, C: traits::tags::Tag<T, Dataset, Attribute>> NxAttribute
    for UnderlyingNexusAttribute<T, C>
{
    #[instrument(skip_all, level = "debug", fields(name = self.name), err(level = "error"))]
    fn create(&mut self, dataset: &Dataset) -> Result<(),CreationError> {
        if self.attribute.is_some() {
            Err(CreationError::AlreadyOpen)
        } else {
            let attribute = self.class.create(dataset, &self.name)?;

            self.attribute = Some(attribute);
            Ok(())
        }
    }

    #[instrument(skip_all, level = "debug", fields(name = self.name), err(level = "error"))]
    fn open(&mut self, parent: &Dataset) -> Result<(),OpeningError> {
        if self.attribute.is_some() {
            Err(OpeningError::AlreadyOpen)
        } else {
            self.attribute = Some(parent.attr(&self.name).map_err(HDF5Error::General)?);
            Ok(())
        }
    }

    #[instrument(skip_all, level = "debug", fields(name = self.name), err(level = "error"))]
    fn close(&mut self) -> Result<(),ClosingError> {
        if self.attribute.is_none() {
            Err(ClosingError::AlreadyClosed)
        } else {
            self.attribute = None;
            Ok(())
        }
    }
}
