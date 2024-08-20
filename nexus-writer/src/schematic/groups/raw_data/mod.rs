use chrono::{DateTime,Utc};
//use data::Data;
use hdf5::{types::{FixedAscii, VarLenAscii}, Attribute, AttributeBuilder, AttributeBuilderData, Dataset, File, Group};
//use instrument::Instrument;
//use periods::Periods;
//use runlog::RunLog;
//use sample::Sample;
//use selog::Selog;
//use user::User;

use crate::schematic::elements::{attribute::NexusAttribute, dataset::{NexusDataset,  NexusUnits, NexusValue}, NexusGroup, NxGroup};

//mod user;
//mod instrument;
//mod sample;
//mod periods;
//mod data;
//mod runlog;
//mod selog;

pub(super) struct RawData {
    idf_version: NexusDataset<u32, 0,
        { NexusUnits::NoUnits },
        { NexusValue::FixedValue }
    >,
    definition: NexusDataset<FixedAscii<6>, 0,
        {[
            AttributeBuilderData::new().create("version"),
            AttributeBuilderData::new().create("URL")
        ]},
        { NexusUnits::NoUnits },
        { NexusValue::FixedValue }
    >,
    definition_local: NexusDataset<FixedAscii<6>,
        "definition_local",
        {[
            AttributeBuilderData::new().create("version"),
            AttributeBuilderData::new().create("URL")
        ]},
        { NexusUnits::NoUnits },
        { NexusValue::FixedValue }
    >,
    program_name: NexusDataset<FixedAscii<6>,
        "program_name",
        {[
            AttributeBuilderData::new().create("version"),
            AttributeBuilderData::new().create("configuration")
        ]},
        { NexusUnits::NoUnits },
        { NexusValue::FixedValue }
    >,
    run_number: NexusDataset<u32,"run_number">,
    title: NexusDataset<VarLenAscii,"title">,
    notes: NexusDataset<VarLenAscii,"notes">,
    start_time: NexusDataset<VarLenAscii,"start_time">,
    end_time: NexusDataset<VarLenAscii,"end_time">,
    duration: NexusDataset<u32,
        "duration",
        {[]},
        { NexusUnits::Second }
    >,
    collection_time: NexusDataset<f64,
        "collection_time",
        {[]},
        { NexusUnits::Second }
    >,
    total_counts: NexusDataset<u32, "total_counts">,
    good_frames: NexusDataset<u32, "good_frames">,
    raw_frames: NexusDataset<u32, "raw_frames">,
    proton_charge: NexusDataset<f64,
        "proton_charge",
        {[]},
        { NexusUnits::MicroAmpHours }
    >,
    experiment_identifier: NexusDataset<VarLenAscii, "experiment_identifier">,
    run_cycle: NexusDataset<VarLenAscii, "run_cycle">,
    //user_1: NexusGroup<User>,
    //run_log: NexusGroup<RunLog>,
    //selog: NexusGroup<Selog>,
    //periods: NexusGroup<Periods>,
    //sample: NexusGroup<Sample>,
    //instrument: NexusGroup<Instrument>,
    //detector_1: NexusGroup<Data>,
}

impl NxGroup for RawData {
    const CLASS_NAME : &'static str = "NXentry";

    fn new() -> Self {
        Self {
            idf_version: NexusDataset::new("idf_version", 2, []),
            definition: NexusDataset::new("definition", FixedAscii::from_ascii("muonTD")),
            run_number: NexusDataset::new("run_number"),
            title: NexusDataset::new("title"),
            notes: NexusDataset::new("notes"),
            start_time: NexusDataset::new("start_time"),
            end_time: NexusDataset::new("end_time"),
            duration: NexusDataset::new("duration"),
            collection_time: NexusDataset::new("collection_time"),
            total_counts: NexusDataset::new("total_counts"),
            good_frames: NexusDataset::new("good_frames"),
            raw_frames: NexusDataset::new("raw_frames"),
            proton_charge: NexusDataset::new("proton_charge"),
            experiment_identifier: NexusDataset::new("experiment_identifier"),
            user_1: NexusGroup::new("user_1"),
            run_log: NexusGroup::new("run_log"),
            selog: NexusGroup::new("selog"),
            periods: NexusGroup::new("periods"),
            sample: NexusGroup::new("sample"),
            instrument: NexusGroup::new("instrument"),
            detector_1: NexusGroup::new("detector_1"),
            definition_local: todo!(),
            program_name: todo!(),
            run_cycle: todo!(),
        }
    }
}
