use hdf5::{types::VarLenAscii, Group};
use supermusr_streaming_types::ecs_f144_logdata_generated::f144_LogData;

use crate::schematic::{
    elements::{
        dataset::NexusDataset,
        group::{self, NexusGroup, NxGroup, NxPushMessageMut, RcGroupContentRegister},
    },
    groups::log::Log,
    nexus_class,
};

pub(super) struct RunLog {
    logs: Vec<NexusGroup<Log>>,
}

impl NxGroup for RunLog {
    const CLASS_NAME: &'static str = nexus_class::RUNLOG;

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            logs: Default::default(),
        }
    }
}

impl<'a> NxPushMessageMut<f144_LogData<'a>> for RunLog {
    type MessageType = f144_LogData<'a>;

    fn push_message_mut(&mut self, message: &Self::MessageType) {
        //self.run_log.push_message(message)
    }
}
