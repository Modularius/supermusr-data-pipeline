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
    nexus::{NexusSettings, RunParameters},
    schematic::{
        elements::{
            attribute::{NexusAttribute, NexusAttributeFixed},
            dataset::{NexusDataset, NexusDatasetFixed},
            group::NexusGroup,
            NexusBuildable, NexusBuilderFinished, NexusDataHolderScalarMutable, NexusDatasetDef,
            NexusError, NexusGroupDef, NexusHandleMessage, NexusHandleMessageWithContext,
            NexusPushMessage, NexusPushMessageWithContext, NexusUnits,
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
                .fixed_value("TODO".parse().expect(""))
                .finish(),
            url: NexusAttribute::begin("URL")
                .fixed_value("TODO".parse().expect(""))
                .finish(),
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

impl NexusGroupDef for RawData {
    const CLASS_NAME: &'static str = nexus_class::ENTRY;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            idf_version: NexusDataset::begin("idf_version").fixed_value(2).finish(),
            definition: NexusDataset::begin("definition")
                .fixed_value("muonTD".parse().expect(""))
                .finish(),
            definition_local: NexusDataset::begin("definition_local")
                .fixed_value("muonTD".parse().expect(""))
                .finish(),
            program_name: NexusDataset::begin("program_name")
                .default_value(Default::default())
                .finish(),
            run_number: NexusDataset::begin("run_number")
                .default_value(Default::default())
                .finish(),
            title: NexusDataset::begin("title")
                .default_value(Default::default())
                .finish(),
            notes: NexusDataset::begin("notes")
                .default_value(Default::default())
                .finish(),
            start_time: NexusDataset::begin("start_time")
                .default_value(Default::default())
                .finish(),
            end_time: NexusDataset::begin("end_time")
                .default_value(Default::default())
                .finish(),
            duration: NexusDataset::begin("duration")
                .default_value(Default::default())
                .finish(),
            collection_time: NexusDataset::begin("collection_time")
                .default_value(Default::default())
                .finish(),
            total_counts: NexusDataset::begin("total_counts")
                .default_value(Default::default())
                .finish(),
            good_frames: NexusDataset::begin("good_frames")
                .default_value(Default::default())
                .finish(),
            raw_frames: NexusDataset::begin("raw_frames")
                .default_value(Default::default())
                .finish(),
            proton_charge: NexusDataset::begin("proton_charge")
                .default_value(Default::default())
                .finish(),
            experiment_identifier: NexusDataset::begin("experiment_identifier")
                .default_value(Default::default())
                .finish(),
            run_cycle: NexusDataset::begin("run_cycle")
                .default_value(Default::default())
                .finish(),
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
    ) -> Result<(), NexusError> {
        run_parameters.num_frames += 1;
        self.detector_1
            .push_message_with_context(message, location, run_parameters)
    }
}

/* Here we handle the start/stop messages */

impl<'a> NexusHandleMessage<RunStart<'a>, Group, RunParameters> for RawData {
    fn handle_message(&mut self, message: &RunStart<'a>, parent: &Group) -> Result<RunParameters, NexusError> {
        self.user_1.push_message(message, parent)?;
        self.periods.push_message(message, parent)?;
        self.sample.push_message(message, parent)?;
        self.instrument.push_message(message, parent)?;

        self.program_name.write_scalar(
            parent,
            "The Program".parse().map_err(|_| NexusError::Unknown)?,
        )?;
        self.run_number.write_scalar(parent, 0)?;
        self.title.write_scalar(
            parent,
            "The Title".parse().map_err(|_| NexusError::Unknown)?,
        )?;
        self.notes.write_scalar(
            parent,
            message
                .metadata()
                .unwrap_or_default()
                .parse()
                .map_err(|_| NexusError::Unknown)?,
        )?;
        self.start_time
            .write_scalar(parent, "Now".parse().map_err(|_| NexusError::Unknown)?)?;
        self.end_time
            .write_scalar(parent, "Then".parse().map_err(|_| NexusError::Unknown)?)?;
        self.duration.write_scalar(parent, 1)?;
        self.collection_time.write_scalar(parent, 1000.0)?;
        self.total_counts.write_scalar(parent, 1)?;
        self.good_frames.write_scalar(parent, 1)?;
        self.raw_frames.write_scalar(parent, 1)?;
        self.proton_charge.write_scalar(parent, 1.0)?;
        self.experiment_identifier
            .write_scalar(parent, "POAS35".parse().map_err(|_| NexusError::Unknown)?)?;
        self.run_cycle
            .write_scalar(parent, "This".parse().map_err(|_| NexusError::Unknown)?)?;

        self.detector_1.push_message(message, parent)?;
        
        Ok(RunParameters::new(message)?)
    }
}

impl<'a> NexusHandleMessage<RunStop<'a>> for RawData {
    fn handle_message(
        &mut self,
        message: &RunStop<'a>,
        location: &Group,
    ) -> Result<(), NexusError> {
        //self.raw_data_1.push_message(message)
        Ok(())
    }
}

/* Here we handle the log messages */

impl<'a> NexusHandleMessage<Alarm<'a>> for RawData {
    fn handle_message(&mut self, message: &Alarm<'a>, parent: &Group) -> Result<(), NexusError> {
        self.selog.push_message(message, parent)
    }
}

impl<'a> NexusHandleMessage<se00_SampleEnvironmentData<'a>> for RawData {
    fn handle_message(
        &mut self,
        message: &se00_SampleEnvironmentData<'a>,
        parent: &Group,
    ) -> Result<(), NexusError> {
        self.selog.push_message(message, parent)
    }
}

impl<'a> NexusHandleMessage<f144_LogData<'a>> for RawData {
    fn handle_message(
        &mut self,
        message: &f144_LogData<'a>,
        parent: &Group,
    ) -> Result<(), NexusError> {
        self.run_log.push_message(message, parent)
    }
}
