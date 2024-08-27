use hdf5::{types::VarLenAscii, Group};
use supermusr_streaming_types::ecs_f144_logdata_generated::f144_LogData;

use crate::schematic::{
    elements::{
        dataset::NexusDataset,
        group::{self, NexusGroup, NxGroup, NxPushMessage, RcGroupContentRegister},
    },
    groups::log::Log,
};

pub(super) struct RunLog {
    logs: Vec<NexusGroup<Log>>,
}

impl NxGroup for RunLog {
    const CLASS_NAME: &'static str = "NXrunlog";

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        Self {
            logs: Default::default(),
        }
    }
}

impl<'a> NxPushMessage<f144_LogData<'a>> for RunLog {
    type MessageType = f144_LogData<'a>;

    fn push_message(&mut self, message: &Self::MessageType) {
        //self.run_log.push_message(message)
    }
}
