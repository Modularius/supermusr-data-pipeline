use chrono::{DateTime, Utc};
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
    error::{NexusConversionError, NexusMissingError, NexusMissingRunStartError, NexusPushError, RunStartError, RunStopError},
    nexus::{NexusSettings, RunBounded, RunStarted},
    schematic::{
        elements::{
            attribute::{NexusAttribute, NexusAttributeFixed},
            dataset::{NexusDataset, NexusDatasetFixed, NexusDatasetMut},
            group::NexusGroup,
            traits::{NexusDataHolderFixed, NexusDataHolderScalarMutable, NexusDataHolderStringMutable, NexusDatasetDef, NexusGroupDef, NexusHandleMessage, NexusPushMessage},
            NexusUnits,
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
            version: NexusAttribute::new_with_fixed_value("version", "TODO".parse().expect("")),
            url: NexusAttribute::new_with_fixed_value("URL", "TODO".parse().expect("")),
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
            idf_version: NexusDataset::new_with_fixed_value("idf_version", 2),
            definition: NexusDataset::new_with_fixed_value("definition", "muonTD".parse().expect("")),
            definition_local: NexusDataset::new_with_fixed_value("definition_local", "muonTD".parse().expect("")),
            program_name: NexusDataset::new_with_auto_default("program_name"),
            run_number: NexusDataset::new_with_auto_default("run_number"),
            title: NexusDataset::new_with_auto_default("title"),
            notes: NexusDataset::new_with_auto_default("notes"),
            start_time: NexusDataset::new_with_auto_default("start_time"),
            end_time: NexusDataset::new_with_auto_default("end_time"),
            duration: NexusDataset::new_with_auto_default("duration"),
            collection_time: NexusDataset::new_with_auto_default("collection_time"),
            total_counts: NexusDataset::new_with_auto_default("total_counts"),
            good_frames: NexusDataset::new_with_auto_default("good_frames"),
            raw_frames: NexusDataset::new_with_auto_default("raw_frames"),
            proton_charge: NexusDataset::new_with_auto_default("proton_charge"),
            experiment_identifier: NexusDataset::new_with_auto_default("experiment_identifier"),
            run_cycle: NexusDataset::new_with_auto_default("run_cycle"),
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

impl<'a> NexusHandleMessage<FrameAssembledEventListMessage<'a>> for RawData {
    fn handle_message(
        &mut self,
        message: &FrameAssembledEventListMessage<'a>,
        location: &Group,
    ) -> Result<(), NexusPushError> {
        self.detector_1
            .push_message(message, location)
    }
}

/* Here we handle the start/stop messages */

impl<'a> NexusHandleMessage<RunStart<'a>, Group, RunStarted> for RawData {
    fn handle_message(
        &mut self,
        message: &RunStart<'a>,
        parent: &Group,
    ) -> Result<RunStarted, NexusPushError> {
        self.user_1.push_message(message, parent)?;
        self.periods.push_message(message, parent)?;
        self.sample.push_message(message, parent)?;
        self.instrument.push_message(message, parent)?;

        self.program_name
            .write_string(parent, "The Program")?;
        self.run_number.write_scalar(parent, 0)?;
        self.title.write_string(parent, "The Title")?;
        self.notes
            .write_string(parent, message.metadata().unwrap_or_default())?;
        self.start_time.write_string(parent, "Now")?;
        self.end_time.write_string(parent, "Then")?;
        self.duration.write_scalar(parent, 1)?;
        self.collection_time.write_scalar(parent, 1000.0)?;
        self.total_counts.write_scalar(parent, 1)?;
        self.good_frames.write_scalar(parent, 1)?;
        self.raw_frames.write_scalar(parent, 1)?;
        self.proton_charge.write_scalar(parent, 1.0)?;
        self.experiment_identifier
            .write_string(parent, "POAS35")?;
        self.run_cycle.write_string(parent, "This")?;

        self.detector_1.push_message(message, parent)?;

        Ok(RunStarted::new(message)?)
    }
}

impl<'a> NexusHandleMessage<RunStop<'a>,Group, DateTime<Utc>> for RawData {
    fn handle_message(
        &mut self,
        message: &RunStop<'a>,
        _location: &Group,
    ) -> Result<DateTime<Utc>, NexusPushError> {
        Ok(DateTime::<Utc>::new(message)?)
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
