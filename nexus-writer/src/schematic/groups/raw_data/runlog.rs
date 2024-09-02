use std::{rc::Rc, sync::Mutex};

use supermusr_streaming_types::ecs_f144_logdata_generated::f144_LogData;

use crate::schematic::{
    elements::group::{GroupContentRegister, NexusGroup, NxGroup, NxPushMessage, NxPushMessageMut},
    elements::traits::GroupBuildable,
    groups::log::Log,
    nexus_class,
};

pub(super) struct RunLog {
    dataset_register: GroupContentRegister,
    logs: Vec<NexusGroup<Log>>,
}

impl NxGroup for RunLog {
    const CLASS_NAME: &'static str = nexus_class::RUNLOG;

    fn new(dataset_register: GroupContentRegister) -> Self {
        Self {
            dataset_register,
            logs: Default::default(),
        }
    }
}

impl<'a> NxPushMessageMut<f144_LogData<'a>> for RunLog {
    type MessageType = f144_LogData<'a>;

    fn push_message_mut(&mut self, message: &Self::MessageType) -> anyhow::Result<()> {
        if let Some(log) = self
            .logs
            .iter()
            .find(|log| log.is_name(message.source_name()))
        {
            log.push_message(message)?;
        } else {
            let log =
                NexusGroup::<Log>::new_subgroup(message.source_name(), &self.dataset_register);
            log.push_message(message)?;
            self.logs.push(log);
        }
        Ok(())
    }
}
