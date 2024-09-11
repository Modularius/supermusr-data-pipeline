use supermusr_streaming_types::ecs_f144_logdata_generated::f144_LogData;

use crate::schematic::{
    elements::{
        group::NexusGroup, NexusError, NexusGroupDef, NexusPushMessage, NexusPushMessageMut,
    },
    groups::log::Log,
    nexus_class,
};

pub(super) struct RunLog {
    logs: Vec<NexusGroup<Log>>,
}

impl NexusGroupDef for RunLog {
    const CLASS_NAME: &'static str = nexus_class::RUNLOG;

    fn new() -> Self {
        Self {
            logs: Default::default(),
        }
    }
}

impl<'a> NexusPushMessageMut<f144_LogData<'a>> for RunLog {
    type MessageType = f144_LogData<'a>;

    fn push_message_mut(&mut self, message: &Self::MessageType) -> Result<(), NexusError> {
        if let Some(log) = self
            .logs
            .iter()
            .find(|log| log.get_name() == message.source_name())
        {
            log.push_message(message)?;
        } else {
            let log = NexusGroup::<Log>::new(message.source_name());
            log.push_message(message)?;
            self.logs.push(log);
        }
        Ok(())
    }
}
