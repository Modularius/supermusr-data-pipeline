use hdf5::{types::VarLenUnicode, Group};
use std::str::FromStr;

use crate::error::{HDF5Error, NexusGroupError, NexusPushError};

use super::traits::{NexusGroupDef, NexusHandleMessage, NexusPushMessage};

pub(crate) struct NexusGroup<D: NexusGroupDef> {
    name: String,
    definition: D,
    group: Option<Group>,
}

impl<D: NexusGroupDef> NexusGroup<D> {
    pub(crate) fn new(name: &str, settings: &D::Settings) -> Self {
        Self {
            name: name.to_string(),
            definition: D::new(settings),
            group: None,
        }
    }
}

impl<D: NexusGroupDef> NexusGroup<D> {
    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }

    pub(in crate::schematic) fn create_hdf5(
        &mut self,
        parent: &Group,
    ) -> Result<Group, NexusGroupError> {
        let group = parent.group(&self.name).or_else(|_| {
            let group = parent.create_group(self.name.as_str())
            .map_err(HDF5Error::HDF5)?;

            group
                .new_attr_builder()
                .with_data(&VarLenUnicode::from_str(D::CLASS_NAME).map_err(HDF5Error::HDF5String)?)
                .create("NXclass").map_err(HDF5Error::HDF5)?;

            Ok::<_, NexusGroupError>(group)
        })?;
        self.group = Some(group.clone());
        Ok(group)
    }

    pub(in crate::schematic) fn close_hdf5(&mut self) {
        self.group = None;
    }
}

impl<D, M, R> NexusPushMessage<M, Group, R> for NexusGroup<D>
where
    D: NexusGroupDef + NexusHandleMessage<M, Group, R>,
{
    fn push_message(&mut self, message: &M, parent: &Group) -> Result<R, NexusPushError> {
        let parent = self.create_hdf5(parent)?;
        let ret = self.definition.handle_message(message, &parent)?;
        self.close_hdf5();
        Ok(ret)
    }
}
