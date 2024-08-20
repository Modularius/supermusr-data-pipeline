mod dataset;
mod group;

use std::{fs::create_dir_all, marker::PhantomData, ops::Deref};
use chrono::{DateTime,Utc};
use dataset::{NexusDatasetData, Units};
use group::{nx_group, NexusGroup};
use hdf5::{plist::DatasetCreateBuilder, Attribute, AttributeBuilder, Dataset, File, Group};

pub(crate) struct NexusFile {
    file: File,
    raw_data_1: NexusGroup<nx_group::RawData1>,
}

impl NexusFile {
    fn new(filename: &str, run_name: &str) {
        create_dir_all(filename)?;
        let filename = {
            let mut filename = filename.to_owned();
            filename.push(run_name);
            filename.set_extension("nxs");
            filename
        };

        let file = File::create(filename)?;
        let raw_data_1 = NexusGroup::new(file);
        Self {
            file,
            raw_data_1 Group::
        }
    }
}

trait TNexusGroup {

}

trait TAttributes {

}

struct WithGroup<N : TNexusGroup, A : TAttributes> {
    from: N,
    group: Group,
}

impl<N : TNexusGroup, A : TAttributes> WithGroup<N,A> {
    fn with_group(self) -> WithGroup<Self, A> {
        WithGroup {
            from: self,
            group: Group::new()
        }
    }
}

struct GroupAnd<G : Group, 

struct ANexusFile {
    file: File,
    raw_data_1: GroupAnd<GroupAnd<GroupAnd<GroupAnd<GroupEnd<
        Group<"Detector1">,
        Group<"Instrument">
        Group<"Periods">
        Group<"Runlog">
        Group<"Selog">
    }


pub(crate) struct RawData1 {
    detector_1: NexusGroup<Detector1>,
    instrument: NexusGroup<Instrument>,
    periods: NexusGroup<Periods>,
    runlog: NexusGroup<Runlog>,
    selog: NexusGroup<Selog>,
    
    idf_version : NexusDataset<NexusDatasetData<i32>>,
    beamline : NexusDataset<NexusDatasetData<String>>,
    collection_time : NexusDataset<NexusDatasetData<f32>, {Units::Second}>,
    definition : NexusDataset<NexusDatasetData<String>>,
    duration : NexusDataset<NexusDatasetData<f32>, {Units::Second}>,
    endtime : NexusDataset<NexusDatasetData<DateTime<Utc>>, {Units::ISO8601}>,
    experiment_identifier : NexusDataset<NexusDatasetData<String>>,
    good_frames : NexusDataset<NexusDatasetData<i32>>,
    name : NexusDataset<NexusDatasetData<String>>,
    notes : NexusDataset<NexusDatasetData<String>>,
    program_name : NexusDataset<NexusDatasetData<String>>,
    proton_charge : NexusDataset<NexusDatasetData<f32>, {Units::UAh}>,
    proton_charge_raw : NexusDataset<NexusDatasetData<f32>, {Units::UAh}>,
    starttime : NexusDataset<NexusDatasetData<DateTime<Utc>>, {Units::ISO8601}>,
}


