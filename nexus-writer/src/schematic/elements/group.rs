use std::str::FromStr;

use hdf5::{types::VarLenUnicode, Dataset, Group, Location};

use super::{NexusError, NexusGroupDef, NexusPushMessage, NexusPushMessageMut};

pub(in crate::schematic) struct NexusGroup<D: NexusGroupDef> {
    name: String,
    definition: D,
    group: Option<Group>,
}

impl<D: NexusGroupDef> NexusGroup<D> {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            definition: D::new(),
            group: None,
        }
    }

    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }
    
    pub(in crate::schematic) fn create_hdf5(&self, parent: &Group) -> Result<Group, NexusError> {
        let group = parent.group(&self.name).or_else(|_| {
            let group = parent
                .create_group(self.name.as_str())
                .map_err(|_| NexusError::Unknown)?;
            
            group
                .new_attr_builder()
                .with_data(&VarLenUnicode::from_str(D::CLASS_NAME).map_err(|_|NexusError::Unknown)?)
                .create("NXclass")
                .map_err(|_|NexusError::Unknown)?;

            Ok(group)
        })?;
        //self.group = Some(group);
        Ok(group)
    }

    pub(in crate::schematic) fn close_hdf5(&mut self) {
        self.group = None;
    }
}

impl<P, D, M> NexusPushMessage<P, M> for NexusGroup<D>
where
    D: NexusGroupDef + NexusPushMessage<P, M>,
{
    fn push_message(&self, message: &M, parent: &P) -> Result<(), NexusError> {
        self.definition.push_message(message, parent)
    }
}

impl<P, D, M> NexusPushMessageMut<P, M> for NexusGroup<D>
where
    D: NexusGroupDef + NexusPushMessageMut<P, M>,
{
    fn push_message_mut(&mut self, message: &M, parent: &P) -> Result<(), NexusError> {
        self.definition.push_message_mut(message, parent)
    }
}
