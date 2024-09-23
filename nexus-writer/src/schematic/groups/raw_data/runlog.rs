use hdf5::{types::TypeDescriptor, Group};
use supermusr_streaming_types::ecs_f144_logdata_generated::{f144_LogData, Value};

use crate::{
    error::NexusPushError,
    nexus::NexusSettings,
    schematic::{
        elements::{group::NexusGroup, NexusGroupDef, NexusHandleMessage, NexusPushMessage},
        groups::log::Log,
        nexus_class,
    },
};

pub(super) struct RunLog {
    settings: NexusSettings,
    logs: Vec<NexusGroup<Log>>,
}

impl<'a> NexusGroupDef for RunLog {
    const CLASS_NAME: &'static str = nexus_class::RUNLOG;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            settings: settings.clone(),
            logs: Default::default(),
        }
    }
}

impl<'a> NexusHandleMessage<f144_LogData<'a>> for RunLog {
    fn handle_message(
        &mut self,
        message: &f144_LogData<'a>,
        location: &Group,
    ) -> Result<(), NexusPushError> {
        if let Some(log) = self
            .logs
            .iter_mut()
            .find(|log| log.get_name() == message.source_name())
        {
            log.push_message(message, location)?;
        } else {
            let mut log = match message.value_type() {
                supermusr_streaming_types::ecs_f144_logdata_generated::Value::Byte => {
                    NexusGroup::<Log>::new(
                        message.source_name(),
                        &(
                            self.settings,
                            TypeDescriptor::Integer(hdf5::types::IntSize::U1),
                        ),
                    )
                }
                supermusr_streaming_types::ecs_f144_logdata_generated::Value::Short => {
                    NexusGroup::<Log>::new(
                        message.source_name(),
                        &(
                            self.settings,
                            TypeDescriptor::Integer(hdf5::types::IntSize::U2),
                        ),
                    )
                }
                supermusr_streaming_types::ecs_f144_logdata_generated::Value::Int => {
                    NexusGroup::<Log>::new(
                        message.source_name(),
                        &(
                            self.settings,
                            TypeDescriptor::Integer(hdf5::types::IntSize::U4),
                        ),
                    )
                }
                supermusr_streaming_types::ecs_f144_logdata_generated::Value::Long => {
                    NexusGroup::<Log>::new(
                        message.source_name(),
                        &(
                            self.settings,
                            TypeDescriptor::Integer(hdf5::types::IntSize::U8),
                        ),
                    )
                }
                supermusr_streaming_types::ecs_f144_logdata_generated::Value::UByte => {
                    NexusGroup::<Log>::new(
                        message.source_name(),
                        &(
                            self.settings,
                            TypeDescriptor::Unsigned(hdf5::types::IntSize::U1),
                        ),
                    )
                }
                supermusr_streaming_types::ecs_f144_logdata_generated::Value::UShort => {
                    NexusGroup::<Log>::new(
                        message.source_name(),
                        &(
                            self.settings,
                            TypeDescriptor::Unsigned(hdf5::types::IntSize::U2),
                        ),
                    )
                }
                supermusr_streaming_types::ecs_f144_logdata_generated::Value::UInt => {
                    NexusGroup::<Log>::new(
                        message.source_name(),
                        &(
                            self.settings,
                            TypeDescriptor::Unsigned(hdf5::types::IntSize::U4),
                        ),
                    )
                }
                supermusr_streaming_types::ecs_f144_logdata_generated::Value::ULong => {
                    NexusGroup::<Log>::new(
                        message.source_name(),
                        &(
                            self.settings,
                            TypeDescriptor::Unsigned(hdf5::types::IntSize::U8),
                        ),
                    )
                }
                supermusr_streaming_types::ecs_f144_logdata_generated::Value::Float => {
                    NexusGroup::<Log>::new(
                        message.source_name(),
                        &(
                            self.settings,
                            TypeDescriptor::Float(hdf5::types::FloatSize::U4),
                        ),
                    )
                }
                supermusr_streaming_types::ecs_f144_logdata_generated::Value::Double => {
                    NexusGroup::<Log>::new(
                        message.source_name(),
                        &(
                            self.settings,
                            TypeDescriptor::Float(hdf5::types::FloatSize::U8),
                        ),
                    )
                }
            };
            log.push_message(message, location)?;
            self.logs.push(log);
        }
        Ok(())
    }
}
