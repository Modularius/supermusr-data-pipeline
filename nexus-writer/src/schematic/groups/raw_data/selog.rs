use hdf5::{Group, Location};
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

impl<'a> NexusPushMessageMut<Group, se00_SampleEnvironmentData<'a>> for Selog {
    fn push_message_mut(&mut self, message: &se00_SampleEnvironmentData<'a>, location: &Location) -> Result<(), NexusError> {
        if let Some(selog) = self.selogs.iter().find(|log| log.get_name() == message.name()) {
            let group = selog.create_hdf5(&location.as_group().expect("Location is Group"))?;
            selog.push_message(message, &group.as_location().expect("Group is Location"))?;
        } else {
            let selog_block = NexusGroup::<SelogBlock>::new(message.name());
            let group = selog_block.create_hdf5(&location.as_group().expect("Location is Group"))?;
            selog_block.push_message(message, &group.as_location().expect("Group is Location"))?;
            self.selogs.push(selog_block);
        }
        Ok(())
    }
}

impl<'a> NexusPushMessageMut<Group, Alarm<'a>> for Selog {
    fn push_message_mut(&mut self, message: &Alarm<'a>, location: &Location) -> Result<(), NexusError> {
        if let Some(selog) = self
            .selogs
            .iter()
            .find(|selog| selog.get_name() == message.source_name().expect(""))
        {
            let group = selog.create_hdf5(&location.as_group().expect("Location is Group"))?;
            selog.push_message(message, &group.as_location().expect("Group is Location"))?;
        } else {
            let selog_block = NexusGroup::<SelogBlock>::new(message.source_name().expect(""));
            let group = selog_block.create_hdf5(&location.as_group().expect("Location is Group"))?;
            selog_block.push_message(message, &group.as_location().expect("Group is Location"))?;
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

impl<'a> NexusPushMessage<Group, se00_SampleEnvironmentData<'a>> for SelogBlock {
    fn push_message(&self, message: &se00_SampleEnvironmentData<'a>, location: &Location) -> Result<(), NexusError> {
        let group = self.value_log.create_hdf5(&location.as_group().expect("Location is Group"))?;
        self.value_log.push_message(message, &group.as_location().expect("Group is Location"))
    }
}

impl<'a> NexusPushMessage<Group, Alarm<'a>> for SelogBlock {
    fn push_message(&self, message: &Alarm<'a>, location: &Location) -> Result<(), NexusError> {
        let group = self.value_log.create_hdf5(&location.as_group().expect("Location is Group"))?;
        self.value_log.push_message(message, &group.as_location().expect("Group is Location"))
    }
}
