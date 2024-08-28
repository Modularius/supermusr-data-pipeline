use supermusr_streaming_types::{
    ecs_al00_alarm_generated::Alarm, ecs_se00_data_generated::se00_SampleEnvironmentData,
};

use crate::schematic::{
    elements::group::{
        NexusGroup, NxGroup, NxPushMessage, NxPushMessageMut, RcGroupContentRegister, RcNexusGroup,
    },
    groups::log::ValueLog,
    nexus_class,
};

pub(super) struct Selog {
    dataset_register: RcGroupContentRegister,
    selogs: Vec<RcNexusGroup<SelogBlock>>,
}

impl NxGroup for Selog {
    const CLASS_NAME: &'static str = nexus_class::SELOG;

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            dataset_register,
            selogs: Default::default(),
        }
    }
}

impl<'a> NxPushMessageMut<se00_SampleEnvironmentData<'a>> for Selog {
    type MessageType = se00_SampleEnvironmentData<'a>;

    fn push_message_mut(&mut self, message: &Self::MessageType) -> anyhow::Result<()> {
        if let Some(selog) = self
            .selogs
            .iter()
            .find(|selog| selog.lock().expect("Lock exists").get_name() == message.name())
        {
            selog.push_message(message);
        } else {
            let selog_block =
                NexusGroup::<SelogBlock>::new(message.name(), Some(self.dataset_register.clone()));
            selog_block.push_message(message);
            self.selogs.push(selog_block);
        }
        Ok(())
    }
}

impl<'a> NxPushMessageMut<Alarm<'a>> for Selog {
    type MessageType = Alarm<'a>;

    fn push_message_mut(&mut self, message: &Self::MessageType) -> anyhow::Result<()> {
        if let Some(selog) = self.selogs.iter().find(|selog| {
            selog.lock().expect("Lock exists").get_name() == message.source_name().expect("")
        }) {
            selog.push_message(message);
        } else {
            let selog_block = NexusGroup::<SelogBlock>::new(
                message.source_name().expect(""),
                Some(self.dataset_register.clone()),
            );
            selog_block.push_message(message);
            self.selogs.push(selog_block);
        }
        Ok(())
    }
}

pub(super) struct SelogBlock {
    value_log: RcNexusGroup<ValueLog>,
}

impl NxGroup for SelogBlock {
    const CLASS_NAME: &'static str = nexus_class::SELOG_BLOCK;

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            value_log: NexusGroup::new("value_log", Some(dataset_register.clone())),
        }
    }
}

impl<'a> NxPushMessage<se00_SampleEnvironmentData<'a>> for SelogBlock {
    type MessageType = se00_SampleEnvironmentData<'a>;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.value_log.push_message(message)
    }
}

impl<'a> NxPushMessage<Alarm<'a>> for SelogBlock {
    type MessageType = Alarm<'a>;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.value_log.push_message(message)
    }
}
