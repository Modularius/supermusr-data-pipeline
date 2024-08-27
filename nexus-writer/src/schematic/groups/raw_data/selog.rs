use hdf5::{types::VarLenAscii, Group};
use supermusr_streaming_types::{
    ecs_al00_alarm_generated::Alarm, ecs_se00_data_generated::se00_SampleEnvironmentData,
};

use crate::schematic::{elements::{
    dataset::{NexusDataset, RcNexusDatasetVar},
    group::{NexusGroup, NxGroup, NxPushMessageMut, RcGroupContentRegister},
}, nexus_class};

pub(super) struct Selog {
    name: RcNexusDatasetVar<VarLenAscii>,
}

impl NxGroup for Selog {
    const CLASS_NAME: &'static str = nexus_class::SELOG;

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            name: NexusDataset::begin().finish("name", dataset_register),
        }
    }
}

impl<'a> NxPushMessageMut<se00_SampleEnvironmentData<'a>> for Selog {
    type MessageType = se00_SampleEnvironmentData<'a>;

    fn push_message_mut(&mut self, message: &Self::MessageType) {
        //self.selog.push_message(message)
    }
}
impl<'a> NxPushMessageMut<Alarm<'a>> for Selog {
    type MessageType = Alarm<'a>;

    fn push_message_mut(&mut self, message: &Self::MessageType) {
        //self.selog.push_message(message)
    }
}
