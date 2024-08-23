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
use supermusr_streaming_types::{aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage, ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm, ecs_f144_logdata_generated::f144_LogData, ecs_pl72_run_start_generated::RunStart, ecs_se00_data_generated::se00_SampleEnvironmentData};
use user::User;

use crate::schematic::elements::{
    attribute::{NexusAttribute, NexusUnits},
    dataset::{MustEnterAttributes, MustEnterFixedValue, NexusDataset, NoAttributesNeeded, RcNexusDatasetFixed, RcNexusDatasetVar},
    group::{NexusGroup, NxGroup, NxPushMessage, RcDatasetRegister},
};

mod data;
mod instrument;
mod periods;
mod runlog;
mod sample;
mod selog;
mod user;

pub(super) struct RawData {
    idf_version: RcNexusDatasetFixed<u32>,
    definition: RcNexusDatasetFixed<FixedAscii<6>, MustEnterAttributes<2>>,
    definition_local: RcNexusDatasetFixed<FixedAscii<6>, MustEnterAttributes<2>>,
    program_name: RcNexusDatasetVar<VarLenAscii>,
    run_number: RcNexusDatasetVar<u32>,
    title: RcNexusDatasetVar<VarLenAscii>,
    notes: RcNexusDatasetVar<VarLenAscii>,
    start_time: RcNexusDatasetVar<VarLenAscii>,
    end_time: RcNexusDatasetVar<VarLenAscii>,
    duration: RcNexusDatasetVar<u32, MustEnterAttributes<1>>,
    collection_time: RcNexusDatasetVar<f64>,
    total_counts: RcNexusDatasetVar<u32>,
    good_frames: RcNexusDatasetVar<u32>,
    raw_frames: RcNexusDatasetVar<u32>,
    proton_charge: RcNexusDatasetVar<f64, MustEnterAttributes<1>>,
    experiment_identifier: RcNexusDatasetVar<VarLenAscii>,
    run_cycle: RcNexusDatasetVar<VarLenAscii>,
    user_1: NexusGroup<User>,
    run_log: NexusGroup<RunLog>,
    selog: NexusGroup<Selog>,
    periods: NexusGroup<Periods>,
    sample: NexusGroup<Sample>,
    instrument: NexusGroup<Instrument>,
    detector_1: NexusGroup<Data>,
}

impl NxGroup for RawData {
    const CLASS_NAME: &'static str = "NXentry";

    fn new(dataset_register : RcDatasetRegister) -> Self {

        //  definition and local_definition
        let definition = NexusDataset::begin()
        .attributes([
            NexusAttribute::new("version", TypeDescriptor::VarLenAscii),
            NexusAttribute::new("URL", TypeDescriptor::VarLenAscii),
        ])
        .fixed_value(FixedAscii::from_ascii("muonTD").expect(""));

        //  program_name
        let program_name = NexusDataset::begin()
            .attributes([
                NexusAttribute::new("version", TypeDescriptor::VarLenAscii),
                NexusAttribute::new("configuration", TypeDescriptor::VarLenAscii),
            ]);

        Self {
            idf_version: NexusDataset::begin().fixed_value(2).finish("idf_version", dataset_register.clone()),
            definition: definition.clone().finish("definition", dataset_register.clone()),
            definition_local: definition.finish("definition_local", dataset_register.clone()),
            program_name: program_name.finish("program_name", dataset_register.clone()),
            run_number: NexusDataset::begin().finish("run_number", dataset_register.clone()),
            title: NexusDataset::begin().finish("title", dataset_register.clone()),
            notes: NexusDataset::begin().finish("notes", dataset_register.clone()),
            start_time: NexusDataset::begin().finish("start_time", dataset_register.clone()),
            end_time: NexusDataset::begin().finish("end_time", dataset_register.clone()),
            duration: NexusDataset::begin()
                .attributes([NexusAttribute::units(NexusUnits::Second)])
                .finish("duration", dataset_register.clone()),
            collection_time: NexusDataset::begin().finish("collection_time", dataset_register.clone()),
            total_counts: NexusDataset::begin().finish("total_counts", dataset_register.clone()),
            good_frames: NexusDataset::begin().finish("good_frames", dataset_register.clone()),
            raw_frames: NexusDataset::begin().finish("raw_frames", dataset_register.clone()),
            proton_charge: NexusDataset::begin().finish("proton_charge", dataset_register.clone()),
            experiment_identifier: NexusDataset::begin().finish("experiment_identifier", dataset_register.clone()),
            run_cycle: NexusDataset::begin().finish("run_cycle", dataset_register.clone()),
            user_1: NexusGroup::new("user_1"),
            run_log: NexusGroup::new("run_log"),
            selog: NexusGroup::new("selog"),
            periods: NexusGroup::new("periods"),
            sample: NexusGroup::new("sample"),
            instrument: NexusGroup::new("instrument"),
            detector_1: NexusGroup::new("detector_1"),
        }
    }
}


impl<'a> NxPushMessage<FrameAssembledEventListMessage<'a>> for RawData {
    type MessageType = FrameAssembledEventListMessage<'a>;

    fn push_message(&mut self, message: &Self::MessageType) {
        self.detector_1.push_message(message)
    }
}

impl<'a> NxPushMessage<RunStart<'a>> for RawData {
    type MessageType = RunStart<'a>;

    fn push_message(&mut self, message: &Self::MessageType) {
        self.user_1.push_message(message);
        self.periods.push_message(message);
        self.sample.push_message(message);
        self.instrument.push_message(message);
    }
}
impl<'a> NxPushMessage<RunStop<'a>> for RawData {
    type MessageType = RunStop<'a>;

    fn push_message(&mut self, message: &Self::MessageType) {
        //self.raw_data_1.push_message(message)
    }
}

impl<'a> NxPushMessage<Alarm<'a>> for RawData {
    type MessageType = Alarm<'a>;

    fn push_message(&mut self, message: &Self::MessageType) {
        self.selog.push_message(message)
    }
}

impl<'a> NxPushMessage<se00_SampleEnvironmentData<'a>> for RawData {
    type MessageType = se00_SampleEnvironmentData<'a>;

    fn push_message(&mut self, message: &Self::MessageType) {
        self.selog.push_message(message)
    }
}

impl<'a> NxPushMessage<f144_LogData<'a>> for RawData {
    type MessageType = f144_LogData<'a>;

    fn push_message(&mut self, message: &Self::MessageType) {
        self.run_log.push_message(message)
    }
}