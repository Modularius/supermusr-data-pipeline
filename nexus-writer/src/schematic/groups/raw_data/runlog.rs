use hdf5::Group;
use supermusr_streaming_types::ecs_f144_logdata_generated::f144_LogData;

use crate::{
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{
        elements::{group::NexusGroup, traits::{NexusGroupDef, NexusHandleMessage, NexusPushMessage}},
        groups::log::Log,
        nexus_class,
    },
};

pub(super) struct RunLog {
    settings: NexusSettings,
    logs: Vec<NexusGroup<Log>>,
}

impl NexusGroupDef for RunLog {
    const CLASS_NAME: &'static str = nexus_class::RUNLOG;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            settings: settings.clone(),
            logs: Default::default(),
        }
    }
}

impl<'a> NexusHandleMessage<f144_LogData<'a>> for RunLog {
    fn handle_message(
        &mut self,
        message: &f144_LogData<'a>,
        location: &Group,
    ) -> Result<(), NexusPushError> {
        if let Some(log) = self
            .logs
            .iter_mut()
            .find(|log| log.get_name() == message.source_name())
        {
            log.push_message(message, location)?;
        } else {
            let mut log = NexusGroup::<Log>::new(message.source_name(), &self.settings);
            log.push_message(message, location)?;
            self.logs.push(log);
        }
        Ok(())
    }
}
