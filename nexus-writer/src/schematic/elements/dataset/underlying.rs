use crate::schematic::elements::{attribute::NxAttribute, traits, traits::Class, NxLivesInGroup};
use hdf5::{Dataset, Group, H5Type};
use tracing::instrument;

use super::{AttributeRegister, NxDataset};

// Implement Database Classes

#[derive(Default)]
pub(crate) struct UnderlyingNexusDataset<
    T,
    D: NxDataset = (),
    C: traits::tags::Tag<T, Group, Dataset> = (),
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
    fn create(&mut self, parent: &Group) -> Result<(), anyhow::Error> {
        if self.dataset.is_some() {
            Err(anyhow::anyhow!("{} dataset already open", self.name))
        } else {
            let dataset = self.class.create(parent, &self.name)?;
            for attribute in self
                .attributes_register
                .lock()
                .iter_mut()
            {
                attribute.lock().expect("Lock Exists").create(&dataset)?;
            }
            self.dataset = Some(dataset);
            Ok(())
        }
    }

    #[instrument(skip_all, level = "debug", fields(name = tracing::field::Empty), err(level = "error"))]
    fn open(&mut self, parent: &Group) -> Result<(), anyhow::Error> {
        if self.dataset.is_some() {
            Err(anyhow::anyhow!("{} dataset already open", self.name))
        } else {
            match parent.dataset(&self.name) {
                Ok(dataset) => {
                    for attribute in self
                        .attributes_register
                        .lock()
                        .iter_mut()
                    {
                        attribute.lock().expect("Lock Exists").open(&dataset)?;
                    }
                    self.dataset = Some(dataset);
                    Ok(())
                }
                Err(e) => Err(e.into()),
            }
        }
    }

    #[instrument(skip_all, level = "debug", fields(name = tracing::field::Empty), err(level = "error"))]
    fn close(&mut self) -> Result<(), anyhow::Error> {
        if self.dataset.is_none() {
            Err(anyhow::anyhow!("{} dataset already closed", self.name))
        } else {
            for attribute in self
                .attributes_register
                .lock()
                .iter_mut()
            {
                attribute.lock().expect("Lock Exists").close()?;
            }
            self.dataset = None;
            Ok(())
        }
    }
}
