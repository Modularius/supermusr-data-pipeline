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
    dataset::{MustEnterAttributes, MustEnterFixedValue, NexusDataset, NoAttributesNeeded},
    group::{NexusGroup, NxGroup, NxPushMessage},
};

mod data;
mod instrument;
mod periods;
mod runlog;
mod sample;
mod selog;
mod user;

pub(super) struct RawData {
    idf_version: NexusDataset<u32, NoAttributesNeeded, MustEnterFixedValue>,
    definition: NexusDataset<FixedAscii<6>, MustEnterAttributes<2>, MustEnterFixedValue>,
    definition_local: NexusDataset<FixedAscii<6>, MustEnterAttributes<2>, MustEnterFixedValue>,
    program_name: NexusDataset<VarLenAscii>,
    run_number: NexusDataset<u32>,
    title: NexusDataset<VarLenAscii>,
    notes: NexusDataset<VarLenAscii>,
    start_time: NexusDataset<VarLenAscii>,
    end_time: NexusDataset<VarLenAscii>,
    duration: NexusDataset<u32, MustEnterAttributes<1>>,
    collection_time: NexusDataset<f64>,
    total_counts: NexusDataset<u32>,
    good_frames: NexusDataset<u32>,
    raw_frames: NexusDataset<u32>,
    proton_charge: NexusDataset<f64, MustEnterAttributes<1>>,
    experiment_identifier: NexusDataset<VarLenAscii>,
    run_cycle: NexusDataset<VarLenAscii>,
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

    fn new() -> Self {

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
            idf_version: NexusDataset::begin().fixed_value(2).finish("idf_version"),
            definition: definition.clone().finish("definition"),
            definition_local: definition.finish("definition_local"),
            program_name: program_name.finish("program_name"),
            run_number: NexusDataset::begin().finish("run_number"),
            title: NexusDataset::begin().finish("title"),
            notes: NexusDataset::begin().finish("notes"),
            start_time: NexusDataset::begin().finish("start_time"),
            end_time: NexusDataset::begin().finish("end_time"),
            duration: NexusDataset::begin()
                .attributes([NexusAttribute::units(NexusUnits::Second)])
                .finish("duration"),
            collection_time: NexusDataset::begin().finish("collection_time"),
            total_counts: NexusDataset::begin().finish("total_counts"),
            good_frames: NexusDataset::begin().finish("good_frames"),
            raw_frames: NexusDataset::begin().finish("raw_frames"),
            proton_charge: NexusDataset::begin().finish("proton_charge"),
            experiment_identifier: NexusDataset::begin().finish("experiment_identifier"),
            run_cycle: NexusDataset::begin().finish("run_cycle"),
            user_1: NexusGroup::new("user_1"),
            run_log: NexusGroup::new("run_log"),
            selog: NexusGroup::new("selog"),
            periods: NexusGroup::new("periods"),
            sample: NexusGroup::new("sample"),
            instrument: NexusGroup::new("instrument"),
            detector_1: NexusGroup::new("detector_1"),
        }
    }

    fn create(&mut self, this: &Group) {
        self.idf_version.create(this);
        self.definition.create(this);
        self.definition_local.create(this);
        self.program_name.create(this);
        self.run_number.create(this);
        self.title.create(this);
        self.notes.create(this);
        self.start_time.create(this);
        self.end_time.create(this);
        self.duration.create(this);
        self.collection_time.create(this);
        self.total_counts.create(this);
        self.good_frames.create(this);
        self.raw_frames.create(this);
        self.proton_charge.create(this);
        self.experiment_identifier.create(this);
        self.run_cycle.create(this);
        self.user_1.create(this);
        self.run_log.create(this);
        self.selog.create(this);
        self.periods.create(this);
        self.sample.create(this);
        self.instrument.create(this);
        self.detector_1.create(this);
    }

    fn open(&mut self, this: &Group) {
        self.idf_version.open(this);
        self.definition.open(this);
        self.definition_local.open(this);
        self.program_name.open(this);
        self.run_number.open(this);
        self.title.open(this);
        self.notes.open(this);
        self.start_time.open(this);
        self.end_time.open(this);
        self.duration.open(this);
        self.collection_time.open(this);
        self.total_counts.open(this);
        self.good_frames.open(this);
        self.raw_frames.open(this);
        self.proton_charge.open(this);
        self.experiment_identifier.open(this);
        self.run_cycle.open(this);
        self.user_1.open(this);
        self.run_log.open(this);
        self.selog.open(this);
        self.periods.open(this);
        self.sample.open(this);
        self.instrument.open(this);
        self.detector_1.open(this);
    }

    fn close(&mut self) {
        self.idf_version.close();
        self.definition.close();
        self.definition_local.close();
        self.program_name.close();
        self.run_number.close();
        self.title.close();
        self.notes.close();
        self.start_time.close();
        self.end_time.close();
        self.duration.close();
        self.collection_time.close();
        self.total_counts.close();
        self.good_frames.close();
        self.raw_frames.close();
        self.proton_charge.close();
        self.experiment_identifier.close();
        self.run_cycle.close();
        self.user_1.close();
        self.run_log.close();
        self.selog.close();
        self.periods.close();
        self.sample.close();
        self.instrument.close();
        self.detector_1.close();
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