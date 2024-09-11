use hdf5::{Group, Location};
use supermusr_streaming_types::ecs_f144_logdata_generated::f144_LogData;

use crate::schematic::{
    elements::{
        group::NexusGroup, NexusError, NexusGroupDef, NexusHandleMessage, NexusPushMessage
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

impl<'a> NexusHandleMessage<f144_LogData<'a>> for RunLog {
    fn handle_message(&mut self, message: &f144_LogData<'a>, location: &Group) -> Result<(), NexusError> {
        if let Some(log) = self
            .logs
            .iter_mut()
            .find(|log| log.get_name() == message.source_name())
        {
            log.push_message(message, location)?;
        } else {
            let mut log = NexusGroup::<Log>::new(message.source_name());
            log.push_message(message, location)?;
            self.logs.push(log);
        }
        Ok(())
    }
}
