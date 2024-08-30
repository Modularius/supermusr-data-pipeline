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
        attribute::{NexusAttribute, NexusAttributeFixed, NexusUnits},
        dataset::{AttributeRegister, NexusDataset, NexusDatasetFixed, NxDataset},
        group::{
            GroupBuildable, GroupContentRegister, NexusGroup, NxGroup, NxPushMessage,
            NxPushMessageMut,
        },
        traits::{Buildable, CanWriteScalar},
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
    version: NexusAttributeFixed<H5String>,
    url: NexusAttributeFixed<H5String>,
}

impl NxDataset for DefinitionAttributes {
    fn new(attribute_register: AttributeRegister) -> Self {
        Self {
            version: NexusAttribute::begin("version")
                .fixed_value("TODO".parse().expect(""))
                .finish(&attribute_register),
            url: NexusAttribute::begin("URL")
                .fixed_value("TODO".parse().expect(""))
                .finish(&attribute_register),
        }
    }
}

#[derive(Clone)]
struct DurationAttributes;
impl NxDataset for DurationAttributes {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Seconds);

    fn new(_attribute_register: AttributeRegister) -> Self {
        Self
    }
}

#[derive(Clone)]
struct ProtonChargeAttributes;

impl NxDataset for ProtonChargeAttributes {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::MicroAmpHours);

    fn new(_attribute_register: AttributeRegister) -> Self {
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
    user_1: NexusGroup<User>,
    run_log: NexusGroup<RunLog>,
    selog: NexusGroup<Selog>,
    periods: NexusGroup<Periods>,
    sample: NexusGroup<Sample>,
    instrument: NexusGroup<Instrument>,
    detector_1: NexusGroup<Data>,
}

impl NxGroup for RawData {
    const CLASS_NAME: &'static str = nexus_class::ENTRY;

    fn new(dataset_register: GroupContentRegister) -> Self {
        Self {
            idf_version: NexusDataset::begin("idf_version")
                .fixed_value(2)
                .finish(&dataset_register),
            definition: NexusDataset::begin("definition")
                .fixed_value("muonTD".parse().expect(""))
                .finish(&dataset_register),
            definition_local: NexusDataset::begin("definition_local")
                .fixed_value("muonTD".parse().expect(""))
                .finish(&dataset_register),
            program_name: NexusDataset::begin("program_name").finish(&dataset_register),
            run_number: NexusDataset::begin("run_number").finish(&dataset_register),
            title: NexusDataset::begin("title").finish(&dataset_register),
            notes: NexusDataset::begin("notes").finish(&dataset_register),
            start_time: NexusDataset::begin("start_time").finish(&dataset_register),
            end_time: NexusDataset::begin("end_time").finish(&dataset_register),
            duration: NexusDataset::begin("duration").finish(&dataset_register),
            collection_time: NexusDataset::begin("collection_time").finish(&dataset_register),
            total_counts: NexusDataset::begin("total_counts").finish(&dataset_register),
            good_frames: NexusDataset::begin("good_frames").finish(&dataset_register),
            raw_frames: NexusDataset::begin("raw_frames").finish(&dataset_register),
            proton_charge: NexusDataset::begin("proton_charge").finish(&dataset_register),
            experiment_identifier: NexusDataset::begin("experiment_identifier")
                .finish(&dataset_register),
            run_cycle: NexusDataset::begin("run_cycle").finish(&dataset_register),
            user_1: NexusGroup::new_subgroup("user_1", &dataset_register),
            run_log: NexusGroup::new_subgroup("run_log", &dataset_register),
            selog: NexusGroup::new_subgroup("selog", &dataset_register),
            periods: NexusGroup::new_subgroup("periods", &dataset_register),
            sample: NexusGroup::new_subgroup("sample", &dataset_register),
            instrument: NexusGroup::new_subgroup("instrument", &dataset_register),
            detector_1: NexusGroup::new_subgroup("detector_1", &dataset_register),
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
