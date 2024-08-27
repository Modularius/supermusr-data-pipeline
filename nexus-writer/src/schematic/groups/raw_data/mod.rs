use data::Data;
use hdf5::{
    types::{FixedAscii, TypeDescriptor, VarLenAscii},
    Group,
};
use instrument::Instrument;
use periods::Periods;
use runlog::RunLog;
use sample::Sample;
use selog::Selog;
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm,
    ecs_f144_logdata_generated::f144_LogData, ecs_pl72_run_start_generated::RunStart,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};
use user::User;

use crate::schematic::{
    elements::{
        attribute::{NexusAttribute, NexusUnits, RcNexusAttributeFixed},
        dataset::{
            CanWriteScalar, NexusDataset, NxContainerAttributes, RcAttributeRegister,
            RcNexusDatasetFixed, RcNexusDatasetVar,
        },
        group::{
            NexusGroup, NxGroup, NxPushMessage, NxPushMessageMut, RcGroupContentRegister,
            RcNexusGroup,
        },
    },
    nexus_class,
};

mod data;
mod instrument;
mod periods;
mod runlog;
mod sample;
mod selog;
mod user;

#[derive(Clone)]
struct DefinitionAttributes {
    version: RcNexusAttributeFixed<VarLenAscii>,
    url: RcNexusAttributeFixed<VarLenAscii>,
}

impl NxContainerAttributes for DefinitionAttributes {
    fn new(attribute_register: RcAttributeRegister) -> Self {
        Self {
            version: NexusAttribute::begin()
                .fixed_value(VarLenAscii::from_ascii("TODO").expect(""))
                .finish("version", attribute_register.clone()),
            url: NexusAttribute::begin()
                .fixed_value(VarLenAscii::from_ascii("TODO").expect(""))
                .finish("URL", attribute_register.clone()),
        }
    }
}

#[derive(Clone)]
struct DurationAttributes;
impl NxContainerAttributes for DurationAttributes {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Seconds);

    fn new(_attribute_register: RcAttributeRegister) -> Self {
        Self
    }
}

#[derive(Clone)]
struct ProtonChargeAttributes;

impl NxContainerAttributes for ProtonChargeAttributes {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::MicroAmpHours);

    fn new(_attribute_register: RcAttributeRegister) -> Self {
        Self
    }
}

pub(super) struct RawData {
    idf_version: RcNexusDatasetFixed<u32>,
    definition: RcNexusDatasetFixed<FixedAscii<6>, DefinitionAttributes>,
    definition_local: RcNexusDatasetFixed<FixedAscii<6>, DefinitionAttributes>,
    program_name: RcNexusDatasetVar<VarLenAscii>,
    run_number: RcNexusDatasetVar<u32>,
    title: RcNexusDatasetVar<VarLenAscii>,
    notes: RcNexusDatasetVar<VarLenAscii>,
    start_time: RcNexusDatasetVar<VarLenAscii>,
    end_time: RcNexusDatasetVar<VarLenAscii>,
    duration: RcNexusDatasetVar<u32, DurationAttributes>,
    collection_time: RcNexusDatasetVar<f64>,
    total_counts: RcNexusDatasetVar<u32>,
    good_frames: RcNexusDatasetVar<u32>,
    raw_frames: RcNexusDatasetVar<u32>,
    proton_charge: RcNexusDatasetVar<f64, ProtonChargeAttributes>,
    experiment_identifier: RcNexusDatasetVar<VarLenAscii>,
    run_cycle: RcNexusDatasetVar<VarLenAscii>,
    user_1: RcNexusGroup<User>,
    run_log: RcNexusGroup<RunLog>,
    selog: RcNexusGroup<Selog>,
    periods: RcNexusGroup<Periods>,
    sample: RcNexusGroup<Sample>,
    instrument: RcNexusGroup<Instrument>,
    detector_1: RcNexusGroup<Data>,
}

impl NxGroup for RawData {
    const CLASS_NAME: &'static str = nexus_class::ENTRY;

    fn new(dataset_register: RcGroupContentRegister) -> Self {
        //  definition and local_definition
        let definition =
            NexusDataset::begin().fixed_value(FixedAscii::from_ascii("muonTD").expect(""));

        //  program_name
        let program_name = NexusDataset::begin();

        Self {
            idf_version: NexusDataset::begin()
                .fixed_value(2)
                .finish("idf_version", dataset_register.clone()),
            definition: definition
                .clone()
                .finish("definition", dataset_register.clone()),
            definition_local: definition.finish("definition_local", dataset_register.clone()),
            program_name: program_name.finish("program_name", dataset_register.clone()),
            run_number: NexusDataset::begin().finish("run_number", dataset_register.clone()),
            title: NexusDataset::begin().finish("title", dataset_register.clone()),
            notes: NexusDataset::begin().finish("notes", dataset_register.clone()),
            start_time: NexusDataset::begin().finish("start_time", dataset_register.clone()),
            end_time: NexusDataset::begin().finish("end_time", dataset_register.clone()),
            duration: NexusDataset::begin().finish("duration", dataset_register.clone()),
            collection_time: NexusDataset::begin()
                .finish("collection_time", dataset_register.clone()),
            total_counts: NexusDataset::begin().finish("total_counts", dataset_register.clone()),
            good_frames: NexusDataset::begin().finish("good_frames", dataset_register.clone()),
            raw_frames: NexusDataset::begin().finish("raw_frames", dataset_register.clone()),
            proton_charge: NexusDataset::begin().finish("proton_charge", dataset_register.clone()),
            experiment_identifier: NexusDataset::begin()
                .finish("experiment_identifier", dataset_register.clone()),
            run_cycle: NexusDataset::begin().finish("run_cycle", dataset_register.clone()),
            user_1: NexusGroup::new("user_1", Some(dataset_register.clone())),
            run_log: NexusGroup::new("run_log", Some(dataset_register.clone())),
            selog: NexusGroup::new("selog", Some(dataset_register.clone())),
            periods: NexusGroup::new("periods", Some(dataset_register.clone())),
            sample: NexusGroup::new("sample", Some(dataset_register.clone())),
            instrument: NexusGroup::new("instrument", Some(dataset_register.clone())),
            detector_1: NexusGroup::new("detector_1", Some(dataset_register.clone())),
        }
    }
}

impl<'a> NxPushMessage<FrameAssembledEventListMessage<'a>> for RawData {
    type MessageType = FrameAssembledEventListMessage<'a>;

    fn push_message(&self, message: &Self::MessageType) {
        self.detector_1.push_message(message)
    }
}

impl<'a> NxPushMessage<RunStart<'a>> for RawData {
    type MessageType = RunStart<'a>;

    fn push_message(&self, message: &Self::MessageType) {
        self.user_1.push_message(message);
        self.periods.push_message(message);
        self.sample.push_message(message);
        self.instrument.push_message(message);

        self.program_name
            .write_scalar(VarLenAscii::from_ascii("The Program").unwrap())
            .expect("");
        self.run_number.write_scalar(0).expect("");
        self.title
            .write_scalar(VarLenAscii::from_ascii("The Title").unwrap())
            .expect("");
        self.notes
            .write_scalar(VarLenAscii::from_ascii(message.metadata().unwrap_or_default()).unwrap())
            .expect("");
        self.start_time
            .write_scalar(VarLenAscii::from_ascii("Now").unwrap())
            .expect("");
        self.end_time
            .write_scalar(VarLenAscii::from_ascii("Then").unwrap())
            .expect("");
        self.duration.write_scalar(1).expect("");
        self.collection_time.write_scalar(1000.0).expect("");
        self.total_counts.write_scalar(1).expect("");
        self.good_frames.write_scalar(1).expect("");
        self.raw_frames.write_scalar(1).expect("");
        self.proton_charge.write_scalar(1.0).expect("");
        self.experiment_identifier
            .write_scalar(VarLenAscii::from_ascii("POAS35").unwrap())
            .expect("");
        self.run_cycle
            .write_scalar(VarLenAscii::from_ascii("This").unwrap())
            .expect("");
    }
}
impl<'a> NxPushMessage<RunStop<'a>> for RawData {
    type MessageType = RunStop<'a>;

    fn push_message(&self, message: &Self::MessageType) {
        //self.raw_data_1.push_message(message)
    }
}

impl<'a> NxPushMessageMut<Alarm<'a>> for RawData {
    type MessageType = Alarm<'a>;

    fn push_message_mut(&mut self, message: &Self::MessageType) {
        self.selog.push_message_mut(message)
    }
}

impl<'a> NxPushMessageMut<se00_SampleEnvironmentData<'a>> for RawData {
    type MessageType = se00_SampleEnvironmentData<'a>;

    fn push_message_mut(&mut self, message: &Self::MessageType) {
        self.selog.push_message_mut(message)
    }
}

impl<'a> NxPushMessageMut<f144_LogData<'a>> for RawData {
    type MessageType = f144_LogData<'a>;

    fn push_message_mut(&mut self, message: &Self::MessageType) {
        self.run_log.push_message_mut(message)
    }
}
