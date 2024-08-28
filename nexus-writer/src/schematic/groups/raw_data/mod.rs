use data::Data;
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
            Buildable, CanWriteScalar, NexusDataset, NexusDatasetFixed, NxContainerAttributes,
            RcAttributeRegister,
        },
        group::{
            NexusGroup, NxGroup, NxPushMessage, NxPushMessageMut, RcGroupContentRegister,
            RcNexusGroup,
        },
    },
    nexus_class, H5String,
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
    version: RcNexusAttributeFixed<H5String>,
    url: RcNexusAttributeFixed<H5String>,
}

impl NxContainerAttributes for DefinitionAttributes {
    fn new(attribute_register: RcAttributeRegister) -> Self {
        Self {
            version: NexusAttribute::begin()
                .fixed_value("TODO".parse().expect(""))
                .finish("version", attribute_register.clone()),
            url: NexusAttribute::begin()
                .fixed_value("TODO".parse().expect(""))
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
    idf_version: NexusDatasetFixed<u32>,
    definition: NexusDatasetFixed<H5String, DefinitionAttributes>,
    definition_local: NexusDatasetFixed<H5String, DefinitionAttributes>,
    program_name: NexusDataset<H5String>,
    run_number: NexusDataset<u32>,
    title: NexusDataset<H5String>,
    notes: NexusDataset<H5String>,
    start_time: NexusDataset<H5String>,
    end_time: NexusDataset<H5String>,
    duration: NexusDataset<u32, DurationAttributes>,
    collection_time: NexusDataset<f64>,
    total_counts: NexusDataset<u32>,
    good_frames: NexusDataset<u32>,
    raw_frames: NexusDataset<u32>,
    proton_charge: NexusDataset<f64, ProtonChargeAttributes>,
    experiment_identifier: NexusDataset<H5String>,
    run_cycle: NexusDataset<H5String>,
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
        let definition = NexusDataset::begin().fixed_value("muonTD".parse().expect(""));

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

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.detector_1.push_message(message)
    }
}

impl<'a> NxPushMessage<RunStart<'a>> for RawData {
    type MessageType = RunStart<'a>;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.user_1.push_message(message)?;
        self.periods.push_message(message)?;
        self.sample.push_message(message)?;
        self.instrument.push_message(message)?;

        self.program_name.write_scalar("The Program".parse()?)?;
        self.run_number.write_scalar(0)?;
        self.title.write_scalar("The Title".parse()?)?;
        self.notes
            .write_scalar(message.metadata().unwrap_or_default().parse()?)?;
        self.start_time.write_scalar("Now".parse()?)?;
        self.end_time.write_scalar("Then".parse()?)?;
        self.duration.write_scalar(1)?;
        self.collection_time.write_scalar(1000.0)?;
        self.total_counts.write_scalar(1)?;
        self.good_frames.write_scalar(1)?;
        self.raw_frames.write_scalar(1)?;
        self.proton_charge.write_scalar(1.0)?;
        self.experiment_identifier.write_scalar("POAS35".parse()?)?;
        self.run_cycle.write_scalar("This".parse()?)?;
        Ok(())
    }
}
impl<'a> NxPushMessage<RunStop<'a>> for RawData {
    type MessageType = RunStop<'a>;

    fn push_message(&self, message: &Self::MessageType) -> anyhow::Result<()> {
        //self.raw_data_1.push_message(message)
        Ok(())
    }
}

impl<'a> NxPushMessageMut<Alarm<'a>> for RawData {
    type MessageType = Alarm<'a>;

    fn push_message_mut(&mut self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.selog.push_message_mut(message)
    }
}

impl<'a> NxPushMessageMut<se00_SampleEnvironmentData<'a>> for RawData {
    type MessageType = se00_SampleEnvironmentData<'a>;

    fn push_message_mut(&mut self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.selog.push_message_mut(message)
    }
}

impl<'a> NxPushMessageMut<f144_LogData<'a>> for RawData {
    type MessageType = f144_LogData<'a>;

    fn push_message_mut(&mut self, message: &Self::MessageType) -> anyhow::Result<()> {
        self.run_log.push_message_mut(message)
    }
}
