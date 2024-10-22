use chrono::{DateTime, Utc};
use data::Data;
use hdf5::{Dataset, Group};
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
    elements::{
        attribute::{NexusAttribute, NexusAttributeFixed, NexusAttributeMut},
        dataset::{NexusDataset, NexusDatasetFixed, NexusDatasetMut},
        group::NexusGroup,
        traits::{
            NexusDataHolderFixed, NexusDataHolderScalarMutable, NexusDataHolderStringMutable, NexusDatasetDef, NexusDatasetDefUnitsOnly, NexusGroupDef, NexusHandleMessage, NexusPushMessage
        },
        NexusUnits,
    },
    error::NexusPushError,
    nexus::{NexusConfiguration, NexusSettings, RunBounded, RunStarted},
    schematic::{nexus_class, H5String},
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
    /// DTD version number
    version: NexusAttributeFixed<H5String>,
    /// URL of XML DTD or schema appropriate for file
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

impl<'a> NexusHandleMessage<RunStart<'a>, Dataset> for DefinitionAttributes {
    fn handle_message(&mut self, _: &RunStart<'a>, parent: &Dataset) -> Result<(), NexusPushError> {
        self.version.write(parent)?;
        self.url.write(parent)?;
        Ok(())
    }
}

#[derive(Default, Clone)]
struct DurationAttributes;
impl NexusDatasetDefUnitsOnly for DurationAttributes {
    const UNITS: NexusUnits = NexusUnits::Seconds;
}

#[derive(Default, Clone)]
struct ProtonChargeAttributes;

impl NexusDatasetDefUnitsOnly for ProtonChargeAttributes {
    const UNITS: NexusUnits = NexusUnits::MicroAmpHours;
}


#[derive(Clone)]
struct ProgramNameAttributes {
    /// version of creating program
    version: NexusAttributeFixed<H5String>,
    /// onfiguration of software e.g. SECI configuration
    configuration: NexusAttributeMut<H5String>,
}

impl NexusDatasetDef for ProgramNameAttributes {
    const UNITS: Option<NexusUnits> = Some(NexusUnits::MicroAmpHours);

    fn new() -> Self {
        Self {
            version: NexusAttribute::new_with_fixed_value("version", "TODO".parse().expect("")),
            configuration: NexusAttribute::new_with_default("URL")
        }
    }
}

impl NexusHandleMessage<NexusConfiguration,Dataset> for ProgramNameAttributes {
    fn handle_message(
        &mut self,
        message: &NexusConfiguration,
        parent: &Dataset,
    ) -> Result<(), NexusPushError> {
        self.version.write(parent)?;
        self.configuration.write_string(parent, message.configuration.as_str())?;
        Ok(())
    }
}

pub(super) struct RawData {
    /// version of IDF that NeXus file confirms to
    idf_version: NexusDatasetFixed<u32>,
    /// the template (DTD name) on which the entry was based, e.g.`muonTD`` (muon, time differential).
    /// It’s suggested that muon definitions always use the prefix ‘muon’, with a subsequent sequence
    /// of capitals defining the unique function of the definition.
    definition: NexusDatasetFixed<H5String, DefinitionAttributes>,
    /// a template (DTD name) on which an extension to the base definition is based
    definition_local: NexusDatasetFixed<H5String, DefinitionAttributes>,
    /// Name of creating program
    program_name: NexusDatasetFixed<H5String,ProgramNameAttributes>,
    /// Run number
    run_number: NexusDatasetMut<u32>,
    /// extended title for the entry, e.g. string containing sample, temperature and field
    title: NexusDatasetMut<H5String>,
    /// log of useful stuff about the experiment, supplied by the user
    notes: NexusDatasetMut<H5String>,
    /// start time and date of measurement
    start_time: NexusDatasetMut<H5String>,
    /// end time and date of measurement
    end_time: NexusDatasetMut<H5String>,
    /// duration of measurement i.e. (end - start)
    duration: NexusDatasetMut<u32, DurationAttributes>,
    /// duration of data collection, taking out periods when collection was suspended
    /// (e.g. because of a beam off or run control veto)
    collection_time: NexusDatasetMut<f64>,
    /// total number of detector events
    total_counts: NexusDatasetMut<u32>,
    /// number of proton pulses used (not vetoed)    
    good_frames: NexusDatasetMut<u32>,
    /// number of proton pulses to target
    raw_frames: NexusDatasetMut<u32>,
    /// experiment number, for ISIS, the RB number
    proton_charge: NexusDatasetMut<f64, ProtonChargeAttributes>,
    /// experiment number, for ISIS, the RB number
    experiment_identifier: NexusDatasetMut<H5String>,
    /// ISIS cycle
    run_cycle: NexusDatasetMut<H5String>,
    /// details of representative user
    user_1: NexusGroup<User>,
    /// 
    run_log: NexusGroup<RunLog>,
    /// 
    selog: NexusGroup<Selog>,
    /// 
    periods: NexusGroup<Periods>,
    /// details of the sample under investigation
    sample: NexusGroup<Sample>,
    /// details of the instrument used
    instrument: NexusGroup<Instrument>,
    /// the data collected
    detector_1: NexusGroup<Data>,
}

impl NexusGroupDef for RawData {
    const CLASS_NAME: &'static str = nexus_class::ENTRY;
    type Settings = NexusSettings;

    fn new(settings: &NexusSettings) -> Self {
        Self {
            idf_version: NexusDataset::new_with_fixed_value("idf_version", 2),
            definition: NexusDataset::new_with_fixed_value(
                "definition",
                "muonTD".parse().expect(""),
            ),
            definition_local: NexusDataset::new_with_fixed_value(
                "definition_local",
                "muonTD".parse().expect(""),
            ),
            program_name: NexusDataset::new_with_fixed_value("program_name", "SuperMuSR Data Pipeline Nexus Writer".parse().expect("")),
            run_number: NexusDataset::new_with_default("run_number"),
            title: NexusDataset::new_with_default("title"),
            notes: NexusDataset::new_with_default("notes"),
            start_time: NexusDataset::new_with_default("start_time"),
            end_time: NexusDataset::new_with_default("end_time"),
            duration: NexusDataset::new_with_default("duration"),
            collection_time: NexusDataset::new_with_default("collection_time"),
            total_counts: NexusDataset::new_with_default("total_counts"),
            good_frames: NexusDataset::new_with_default("good_frames"),
            raw_frames: NexusDataset::new_with_default("raw_frames"),
            proton_charge: NexusDataset::new_with_default("proton_charge"),
            experiment_identifier: NexusDataset::new_with_default("experiment_identifier"),
            run_cycle: NexusDataset::new_with_default("run_cycle"),
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
        self.periods.push_message(message, location)?;
        self.detector_1.push_message(message, location)?;
        Ok(())
    }
}

/* Here we handle the start/stop messages */

impl NexusHandleMessage<NexusConfiguration> for RawData {
    fn handle_message(
        &mut self,
        message: &NexusConfiguration,
        parent: &Group,
    ) -> Result<(), NexusPushError> {
        self.title.write_string(parent, "The Title")?;
        self.experiment_identifier.write_string(parent, "POAS35")?;
        self.program_name.push_message(message, parent)
    }
}

impl<'a> NexusHandleMessage<RunStart<'a>, Group, RunStarted> for RawData {
    fn handle_message(
        &mut self,
        message: &RunStart<'a>,
        parent: &Group,
    ) -> Result<RunStarted, NexusPushError> {
        self.idf_version.write(parent)?;
        self.definition.push_message(message, parent)?;
        self.definition_local.push_message(message, parent)?;
        
        self.run_number.write_scalar(parent, 0)?;
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
        self.run_cycle.write_string(parent, "This")?;
        
        self.user_1.push_message(message, parent)?;
        self.sample.push_message(message, parent)?;
        self.instrument.push_message(message, parent)?;

        self.detector_1.push_message(message, parent)?;

        Ok(RunStarted::new(message)?)
    }
}

impl<'a> NexusHandleMessage<RunStop<'a>, Group, DateTime<Utc>> for RawData {
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
