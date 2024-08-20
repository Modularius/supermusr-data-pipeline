use hdf5::Group;

pub(crate) enum ClassName {
    Entry,
    EventData,
    Instrument,
    Detector,
    Source,
    Period,
    Runlog,
    Log,
    Selog,
    Seblock,
}

impl ClassName {
    fn get(&self) -> &'static str {
        match self {
            ClassName::Entry => "NXentry",
            ClassName::EventData => "NXevent_data",
            ClassName::Instrument => "NXinstrument",
            ClassName::Detector => "NXdetector",
            ClassName::Source => "NXsource",
            ClassName::Period => "NXperiod",
            ClassName::Runlog => "NXrunlog",
            ClassName::Log => "NXlog",
            ClassName::Selog => "IXselog",
            ClassName::Seblock => "IXseblock",
        }
    }
}

pub(crate) trait NxGroup {
    const NAME: &'static str;
    const CLASS: ClassName;

    fn create(parent: Group) -> Group {
        let group = parent.create_group(Self::NAME);
        group
            .new_attr_builder()
            .with_data(Self::CLASS.get())
            .create("NX_class")
            .unwrap();
        group
    }

    fn new(parent: Group) -> Self;
}

pub(crate) trait NxBlock {
    const CLASS: &'static str;

    fn create(parent: Group, name: &str) -> Group {
        let group = parent
            .create_group(name);
    }
}

pub(crate) mod nx_group {
    use chrono::{DateTime, Utc};
    use hdf5::Group;

    use crate::schematic::{dataset::{NexusDatasetData, Units}, NexusDataset};

    use super::{ClassName, NexusGroup, NxGroup};

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

    impl NxGroup for RawData1 {
        const CLASS: ClassName = ClassName::Entry;
        const NAME: &'static str = "raw_data_1";

        fn new(this : Group) -> Self {
            Self {
                // Groups
                detector_1: NexusGroup::new(this),
                instrument: NexusGroup::new(this),
                periods: NexusGroup::new(this),
                runlog: NexusGroup::new(this),
                selog: NexusGroup::new(this),
                // Datasets
                idf_version : NexusDataset::new(this),
                beamline : NexusDataset::new(this),
                collection_time : NexusDataset::new(this),
                definition : NexusDataset::new(this),
                duration : NexusDataset::new(this),
                endtime : NexusDataset::new(this),
                experiment_identifier : NexusDataset::new(this),
                good_frames : NexusDataset::new(this),
                name : NexusDataset::new(this),
                notes : NexusDataset::new(this),
                program_name : NexusDataset::new(this),
                proton_charge : NexusDataset::new(this),
                proton_charge_raw : NexusDataset::new(this),
                starttime : NexusDataset::new(this),
            }
        }
    }

    pub(crate) struct Detector1 {}

    pub(crate) struct Instrument {
        detector: NexusGroup<Instrument>,
        source: NexusGroup<Periods>,
    }

    impl NxGroup for Instrument {
        const CLASS: ClassName = ClassName::Instrument;
        const NAME: &'static str = "instrument";
        
        fn new(this : Group) -> Self {
            Self {
                detector: NexusGroup::new(this),
                source: NexusGroup::new(this)
            }
        }
    }

    pub(crate) struct Detector {}
    
    impl NxGroup for Detector {
        const CLASS: ClassName = ClassName::Detector;
        const NAME: &'static str = "detector";
        
        fn new(this : Group) -> Self {
            Self {}
        }
    }

    impl NxGroup for Detector1 {
        const CLASS: ClassName = ClassName::EventData;
        const NAME: &'static str = "detector_1";

        fn new(this : Group) -> Self {
            Self {}
        }
    }

    pub(crate) struct Source {}

    impl NxGroup for Source {
        const CLASS: ClassName = ClassName::Source;
        const NAME: &'static str = "source";
        
        fn new(this : Group) -> Self {
            Self {}
        }
    }

    pub(crate) struct Periods {}

    impl NxGroup for Periods {
        const CLASS: ClassName = ClassName::Period;
        const NAME: &'static str = "periods";
        
        fn new(this : Group) -> Self {
            Self {}
        }
    }

    pub(crate) struct Runlog {}

    impl NxGroup for Runlog {
        const CLASS: ClassName = ClassName::Runlog;
        const NAME: &'static str = "runlog";
        
        fn new(this : Group) -> Self {
            Self {}
        }
    }
    pub(crate) struct Selog {}

    impl NxGroup for Selog {
        const CLASS: ClassName = ClassName::Selog;
        const NAME: &'static str = "selog";
        
        fn new(this : Group) -> Self {
            Self {}
        }
    }

    pub(crate) struct ValueLog {}

    impl NxGroup for ValueLog {
        const CLASS: ClassName = ClassName::Log;
        const NAME: &'static str = "value_log";
        
        fn new(this : Group) -> Self {
            Self {}
        }
    }
}

pub(crate) struct NexusGroup<G : NxGroup> {
    group: Group,
    structure : G
}

impl<G : NxGroup> NexusGroup<G> {
    pub(crate) fn new(parent: Group) -> Self {
        let group =  G::create(parent);
        Self {
            group,
            structure: G::new(group)
        }
    }
}