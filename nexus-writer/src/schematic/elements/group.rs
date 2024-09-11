use super::{NexusError, NexusGroupDef, NexusPushMessage, NexusPushMessageMut};

pub(crate) struct NexusGroup<D: NexusGroupDef> {
    name: String,
    definition: D,
}

impl<D: NexusGroupDef> NexusGroup<D> {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            definition: D::new(),
        }
    }

    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }
}

impl<D, M> NexusPushMessage<M> for NexusGroup<D>
where
    D: NexusGroupDef + NexusPushMessage<M, MessageType = M>,
{
    type MessageType = M;

    fn push_message(&self, message: &Self::MessageType) -> Result<(), NexusError> {
        self.definition.push_message(message)
    }
}

impl<D, M> NexusPushMessageMut<M> for NexusGroup<D>
where
    D: NexusGroupDef + NexusPushMessageMut<M, MessageType = M>,
{
    type MessageType = M;

    fn push_message_mut(&mut self, message: &Self::MessageType) -> Result<(), NexusError> {
        self.definition.push_message_mut(message)
    }
}
