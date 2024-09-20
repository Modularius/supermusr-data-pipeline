use hdf5::Group;
use supermusr_streaming_types::{
    ecs_al00_alarm_generated::Alarm, ecs_se00_data_generated::se00_SampleEnvironmentData,
};

use crate::{
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{
        elements::{group::NexusGroup, NexusGroupDef, NexusHandleMessage, NexusPushMessage},
        groups::log::ValueLog,
        nexus_class,
    },
};

pub(super) struct Selog {
    settings: NexusSettings,
    selogs: Vec<NexusGroup<SelogBlock>>,
}

impl NexusGroupDef for Selog {
    const CLASS_NAME: &'static str = nexus_class::SELOG;

    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            settings: settings.clone(),
            selogs: Default::default(),
        }
    }
}

impl<'a> NexusHandleMessage<se00_SampleEnvironmentData<'a>> for Selog {
    fn handle_message(
        &mut self,
        message: &se00_SampleEnvironmentData<'a>,
        location: &Group,
    ) -> Result<(), NexusPushError> {
        if let Some(selog_block) = self
            .selogs
            .iter_mut()
            .find(|selog_block| selog_block.get_name() == message.name())
        {
            let group = selog_block.create_hdf5(location)?;
            selog_block.push_message(message, &group)?;
        } else {
            let mut selog_block = NexusGroup::<SelogBlock>::new(message.name(), &self.settings);
            let group = selog_block.create_hdf5(location)?;
            selog_block.push_message(message, &group)?;
            self.selogs.push(selog_block);
        }
        Ok(())
    }
}

impl<'a> NexusHandleMessage<Alarm<'a>> for Selog {
    fn handle_message(
        &mut self,
        message: &Alarm<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        if let Some(selog) = self
            .selogs
            .iter_mut()
            .find(|selog| selog.get_name() == message.source_name().expect(""))
        {
            //let group = selog.create_hdf5(location)?;
            selog.push_message(message, parent)?;
        } else {
            let mut selog_block =
                NexusGroup::<SelogBlock>::new(message.source_name().expect(""), &self.settings);
            //let group = selog_block.create_hdf5(parent)?;
            selog_block.push_message(message, parent)?;
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
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            value_log: NexusGroup::new("value_log", settings),
        }
    }
}

impl<'a> NexusHandleMessage<se00_SampleEnvironmentData<'a>> for SelogBlock {
    fn handle_message(
        &mut self,
        message: &se00_SampleEnvironmentData<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        //let group = self.value_log.create_hdf5(location)?;
        self.value_log.push_message(message, &parent)
    }
}

impl<'a> NexusHandleMessage<Alarm<'a>> for SelogBlock {
    fn handle_message(
        &mut self,
        message: &Alarm<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        //let group = self.value_log.create_hdf5(location)?;
        self.value_log.push_message(message, parent)
    }
}
