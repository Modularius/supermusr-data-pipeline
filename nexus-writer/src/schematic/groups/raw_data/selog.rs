use supermusr_streaming_types::{
    ecs_al00_alarm_generated::Alarm, ecs_se00_data_generated::se00_SampleEnvironmentData,
};

use crate::schematic::{
    elements::{
        group::NexusGroup, NexusError, NexusGroupDef, NexusPushMessage, NexusPushMessageMut,
    },
    groups::log::ValueLog,
    nexus_class,
};

pub(super) struct Selog {
    selogs: Vec<NexusGroup<SelogBlock>>,
}

impl NexusGroupDef for Selog {
    const CLASS_NAME: &'static str = nexus_class::SELOG;

    fn new() -> Self {
        Self {
            selogs: Default::default(),
        }
    }
}

impl<'a> NexusPushMessageMut<se00_SampleEnvironmentData<'a>> for Selog {
    type MessageType = se00_SampleEnvironmentData<'a>;

    fn push_message_mut(&mut self, message: &Self::MessageType) -> Result<(), NexusError> {
        if let Some(selog) = self.selogs.iter().find(|log| log.get_name() == message.name()) {
            selog.push_message(message)?;
        } else {
            let selog_block = NexusGroup::<SelogBlock>::new(message.name());
            selog_block.push_message(message)?;
            self.selogs.push(selog_block);
        }
        Ok(())
    }
}

impl<'a> NexusPushMessageMut<Alarm<'a>> for Selog {
    type MessageType = Alarm<'a>;

    fn push_message_mut(&mut self, message: &Self::MessageType) -> Result<(), NexusError> {
        if let Some(selog) = self
            .selogs
            .iter()
            .find(|selog| selog.get_name() == message.source_name().expect(""))
        {
            selog.push_message(message)?;
        } else {
            let selog_block = NexusGroup::<SelogBlock>::new(message.source_name().expect(""));
            selog_block.push_message(message)?;
            self.selogs.push(selog_block);
        }
        Ok(())
    }
}

pub(super) struct SelogBlock {
    value_log: NexusGroup<ValueLog>,
}

impl NexusGroupDef for SelogBlock {
    const CLASS_NAME: &'static str = nexus_class::SELOG_BLOCK;

    fn new() -> Self {
        Self {
            value_log: NexusGroup::new("value_log"),
        }
    }
}

impl<'a> NexusPushMessage<se00_SampleEnvironmentData<'a>> for SelogBlock {
    type MessageType = se00_SampleEnvironmentData<'a>;

    fn push_message(&self, message: &Self::MessageType) -> Result<(), NexusError> {
        self.value_log.push_message(message)
    }
}

impl<'a> NexusPushMessage<Alarm<'a>> for SelogBlock {
    type MessageType = Alarm<'a>;

    fn push_message(&self, message: &Self::MessageType) -> Result<(), NexusError> {
        self.value_log.push_message(message)
    }
}
