use std::str::FromStr;

use hdf5::{types::VarLenUnicode, Group, Location, Object};

use super::{NexusError, NexusGroupDef, NexusHandleMessage, NexusPushMessage};

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
    
    pub(in crate::schematic) fn create_hdf5(&mut self, parent: &Group) -> Result<Group, NexusError> {
        let group = parent.group(&self.name).or_else(|_| {
            let group = parent
                .create_group(self.name.as_str())?;
            
            group
                .new_attr_builder()
                .with_data(&VarLenUnicode::from_str(D::CLASS_NAME).map_err(|_|NexusError::Unknown)?)
                .create("NXclass")?;

            Ok::<_,NexusError>(group)
        })?;
        self.group = Some(group.clone());
        Ok(group)
    }

    pub(in crate::schematic) fn close_hdf5(&mut self) {
        self.group = None;
    }
}

impl<D, M> NexusPushMessage<M,Group> for NexusGroup<D>
where
    D: NexusGroupDef + NexusHandleMessage<M,Group>,
{
    fn push_message(&mut self, message: &M, parent: &Group) -> Result<(), NexusError> {
        let parent = self.create_hdf5(parent)?;
        self.definition.handle_message(message, &parent)?;
        self.close_hdf5();
        Ok(())
    }
}