use hdf5::{Attribute, Dataset, H5Type};
use tracing::instrument;

use super::NxAttribute;
use crate::schematic::elements::traits::{self, Class};

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
    fn create(&mut self, dataset: &Dataset) -> anyhow::Result<()> {
        if self.attribute.is_some() {
            Err(anyhow::anyhow!("{} attribute already open", self.name))
        } else {
            let attribute = self.class.create(dataset, &self.name)?;

            self.attribute = Some(attribute);
            Ok(())
        }
    }

    #[instrument(skip_all, level = "debug", fields(name = self.name), err(level = "error"))]
    fn open(&mut self, parent: &Dataset) -> anyhow::Result<()> {
        if self.attribute.is_some() {
            Err(anyhow::anyhow!("{} attribute already open", self.name))
        } else {
            self.attribute = Some(parent.attr(&self.name)?);
            Ok(())
        }
    }

    #[instrument(skip_all, level = "debug", fields(name = self.name), err(level = "error"))]
    fn close(&mut self) -> anyhow::Result<()> {
        if self.attribute.is_none() {
            Err(anyhow::anyhow!("{} attribute already closed", self.name))
        } else {
            self.attribute = None;
            Ok(())
        }
    }
}
