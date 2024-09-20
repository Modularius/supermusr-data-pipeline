use data::Data;
use hdf5::Group;
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

use crate::{
    error::NexusPushError,
    nexus::{NexusSettings, RunParameters},
    schematic::{
        elements::{
            attribute::{NexusAttribute, NexusAttributeFixed},
            dataset::{NexusDataset, NexusDatasetFixed, NexusDatasetMut},
            group::NexusGroup,
            NexusBuildable, NexusDataHolderScalarMutable, NexusDatasetDef, NexusGroupDef,
            NexusHandleMessage, NexusHandleMessageWithContext, NexusPushMessage,
            NexusPushMessageWithContext, NexusUnits,
        },
        nexus_class, H5String,
    },
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

impl NexusDatasetDef for DefinitionAttributes {
    fn new() -> Self {
        Self {
            version: NexusAttribute::begin("version")
                .finish_with_fixed_value("TODO".parse().expect("")),
            url: NexusAttribute::begin("URL").finish_with_fixed_value("TODO".parse().expect("")),
        }
    }
}

#[derive(Clone)]
struct DurationAttributes;
impl NexusDatasetDef for DurationAttributes {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::Seconds);

    fn new() -> Self {
        Self
    }
}

#[derive(Clone)]
struct ProtonChargeAttributes;

impl NexusDatasetDef for ProtonChargeAttributes {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::MicroAmpHours);

    fn new() -> Self {
        Self
    }
}

pub(super) struct RawData {
    idf_version: NexusDatasetFixed<u32>,
    definition: NexusDatasetFixed<H5String, DefinitionAttributes>,
    definition_local: NexusDatasetFixed<H5String, DefinitionAttributes>,
    program_name: NexusDatasetMut<H5String>,
    run_number: NexusDatasetMut<u32>,
    title: NexusDatasetMut<H5String>,
    notes: NexusDatasetMut<H5String>,
    start_time: NexusDatasetMut<H5String>,
    end_time: NexusDatasetMut<H5String>,
    duration: NexusDatasetMut<u32, DurationAttributes>,
    collection_time: NexusDatasetMut<f64>,
    total_counts: NexusDatasetMut<u32>,
    good_frames: NexusDatasetMut<u32>,
    raw_frames: NexusDatasetMut<u32>,
    proton_charge: NexusDatasetMut<f64, ProtonChargeAttributes>,
    experiment_identifier: NexusDatasetMut<H5String>,
    run_cycle: NexusDatasetMut<H5String>,
    user_1: NexusGroup<User>,
    run_log: NexusGroup<RunLog>,
    selog: NexusGroup<Selog>,
    periods: NexusGroup<Periods>,
    sample: NexusGroup<Sample>,
    instrument: NexusGroup<Instrument>,
    detector_1: NexusGroup<Data>,
}

impl NexusGroupDef for RawData {
    const CLASS_NAME: &'static str = nexus_class::ENTRY;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            idf_version: NexusDataset::begin("idf_version").finish_with_fixed_value(2),
            definition: NexusDataset::begin("definition")
                .finish_with_fixed_value("muonTD".parse().expect("")),
            definition_local: NexusDataset::begin("definition_local")
                .finish_with_fixed_value("muonTD".parse().expect("")),
            program_name: NexusDataset::begin("program_name").finish_with_auto_default(),
            run_number: NexusDataset::begin("run_number").finish_with_auto_default(),
            title: NexusDataset::begin("title").finish_with_auto_default(),
            notes: NexusDataset::begin("notes").finish_with_auto_default(),
            start_time: NexusDataset::begin("start_time").finish_with_auto_default(),
            end_time: NexusDataset::begin("end_time").finish_with_auto_default(),
            duration: NexusDataset::begin("duration").finish_with_auto_default(),
            collection_time: NexusDataset::begin("collection_time").finish_with_auto_default(),
            total_counts: NexusDataset::begin("total_counts").finish_with_auto_default(),
            good_frames: NexusDataset::begin("good_frames").finish_with_auto_default(),
            raw_frames: NexusDataset::begin("raw_frames").finish_with_auto_default(),
            proton_charge: NexusDataset::begin("proton_charge").finish_with_auto_default(),
            experiment_identifier: NexusDataset::begin("experiment_identifier")
                .finish_with_auto_default(),
            run_cycle: NexusDataset::begin("run_cycle").finish_with_auto_default(),
            user_1: NexusGroup::new("user_1", settings),
            run_log: NexusGroup::new("run_log", settings),
            selog: NexusGroup::new("selog", settings),
            periods: NexusGroup::new("periods", settings),
            sample: NexusGroup::new("sample", settings),
            instrument: NexusGroup::new("instrument", settings),
            detector_1: NexusGroup::new("detector_1", settings),
        }
    }
}

/* Here we handle the frame eventlist messages
We also alter the RunParameters context*/

impl<'a> NexusHandleMessageWithContext<FrameAssembledEventListMessage<'a>> for RawData {
    type Context = RunParameters;

    fn handle_message_with_context(
        &mut self,
        message: &FrameAssembledEventListMessage<'a>,
        location: &Group,
        run_parameters: &mut RunParameters,
    ) -> Result<(), NexusPushError> {
        run_parameters.num_frames += 1;
        self.detector_1
            .push_message_with_context(message, location, run_parameters)
    }
}

/* Here we handle the start/stop messages */

impl<'a> NexusHandleMessage<RunStart<'a>, Group, RunParameters> for RawData {
    fn handle_message(
        &mut self,
        message: &RunStart<'a>,
        parent: &Group,
    ) -> Result<RunParameters, NexusPushError> {
        self.user_1.push_message(message, parent)?;
        self.periods.push_message(message, parent)?;
        self.sample.push_message(message, parent)?;
        self.instrument.push_message(message, parent)?;

        self.program_name
            .write_scalar(parent, "The Program".parse()?)?;
        self.run_number.write_scalar(parent, 0)?;
        self.title.write_scalar(parent, "The Title".parse()?)?;
        self.notes
            .write_scalar(parent, message.metadata().unwrap_or_default().parse()?)?;
        self.start_time.write_scalar(parent, "Now".parse()?)?;
        self.end_time.write_scalar(parent, "Then".parse()?)?;
        self.duration.write_scalar(parent, 1)?;
        self.collection_time.write_scalar(parent, 1000.0)?;
        self.total_counts.write_scalar(parent, 1)?;
        self.good_frames.write_scalar(parent, 1)?;
        self.raw_frames.write_scalar(parent, 1)?;
        self.proton_charge.write_scalar(parent, 1.0)?;
        self.experiment_identifier
            .write_scalar(parent, "POAS35".parse()?)?;
        self.run_cycle.write_scalar(parent, "This".parse()?)?;

        self.detector_1.push_message(message, parent)?;

        Ok(RunParameters::new(message)?)
    }
}

impl<'a> NexusHandleMessage<RunStop<'a>> for RawData {
    fn handle_message(
        &mut self,
        message: &RunStop<'a>,
        location: &Group,
    ) -> Result<(), NexusPushError> {
        //self.raw_data_1.push_message(message)
        Ok(())
    }
}

/* Here we handle the log messages */

impl<'a> NexusHandleMessage<Alarm<'a>> for RawData {
    fn handle_message(
        &mut self,
        message: &Alarm<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        self.selog.push_message(message, parent)
    }
}

impl<'a> NexusHandleMessage<se00_SampleEnvironmentData<'a>> for RawData {
    fn handle_message(
        &mut self,
        message: &se00_SampleEnvironmentData<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        self.selog.push_message(message, parent)
    }
}

impl<'a> NexusHandleMessage<f144_LogData<'a>> for RawData {
    fn handle_message(
        &mut self,
        message: &f144_LogData<'a>,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        self.run_log.push_message(message, parent)
    }
}
