use hdf5::{types::VarLenAscii, Group};
use supermusr_streaming_types::{
    ecs_al00_alarm_generated::Alarm, ecs_se00_data_generated::se00_SampleEnvironmentData,
};

use crate::schematic::elements::{
    dataset::{NexusDataset, RcNexusDatasetVar},
    group::{NexusGroup, NxGroup, NxPushMessage, RcGroupContentRegister},
};

pub(super) struct Selog {
    name: RcNexusDatasetVar<VarLenAscii>,
}

impl NxGroup for Selog {
    const CLASS_NAME: &'static str = "NXselog";

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            name: NexusDataset::begin().finish("name", dataset_register),
        }
    }
}

impl<'a> NxPushMessage<se00_SampleEnvironmentData<'a>> for Selog {
    type MessageType = se00_SampleEnvironmentData<'a>;

    fn push_message(&mut self, message: &Self::MessageType) {
        //self.selog.push_message(message)
    }
}
impl<'a> NxPushMessage<Alarm<'a>> for Selog {
    type MessageType = Alarm<'a>;

    fn push_message(&mut self, message: &Self::MessageType) {
        //self.selog.push_message(message)
    }
}
