use hdf5::{types::VarLenAscii, Group};
use supermusr_streaming_types::{ecs_al00_alarm_generated::Alarm, ecs_se00_data_generated::se00_SampleEnvironmentData};

use crate::schematic::elements::{
    dataset::NexusDataset,
    group::{NexusGroup, NxGroup, NxPushMessage},
};

pub(super) struct Selog {
    name: NexusDataset<VarLenAscii>,
}

impl NxGroup for Selog {
    const CLASS_NAME: &'static str = "NXperiod";

    fn new() -> Self {
        Self {
            name: NexusDataset::begin().finish(""),
        }
    }

    fn create(&mut self, this: &Group) {
        self.name.create(this);
    }

    fn open(&mut self, this: &Group) {
        self.name.open(this);
    }

    fn close(&mut self) {
        self.name.close();
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