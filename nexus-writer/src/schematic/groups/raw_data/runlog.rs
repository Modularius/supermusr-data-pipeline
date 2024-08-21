use hdf5::{types::VarLenAscii, Group};
use supermusr_streaming_types::ecs_f144_logdata_generated::f144_LogData;

use crate::schematic::{elements::{
    dataset::NexusDataset,
    group::{self, NexusGroup, NxGroup, NxPushMessage},
}, groups::log::Log};

pub(super) struct RunLog {
    logs: Vec<NexusGroup<Log>>,
}

impl NxGroup for RunLog {
    const CLASS_NAME: &'static str = "NXrunlog";

    fn new() -> Self {
        Self {
            logs: Default::default(),
        }
    }

    fn create(&mut self, this: &Group) {
        
    }

    fn open(&mut self, this: &Group) {
        self.logs = this.groups().expect("").iter().map(|group| {
            let mut nexus_group = NexusGroup::<Log>::new(&group.name());
            nexus_group.open(&group);
            nexus_group
        }).collect();
    }

    fn close(&mut self) {
        for log in &mut self.logs {
            log.close();
        }
    }
}

impl<'a> NxPushMessage<f144_LogData<'a>> for RunLog {
    type MessageType = f144_LogData<'a>;

    fn push_message(&mut self, message: &Self::MessageType) {
        //self.run_log.push_message(message)
    }
}