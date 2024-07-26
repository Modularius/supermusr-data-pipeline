use std::ops::Deref;

use chrono::{DateTime, Utc};

pub(crate) enum Units {
    Second,
    Microsecond,
    ISO8601,
    Mev,
    UAh,
    Counts
}

impl Units {
    fn get(&self) -> &'static str {
        match self {
            Self::Second => "second",
            Self::Microsecond => "microsecond",
            Self::ISO8601 => "ISO8601",
            Self::Mev => "mEv",
            Self::UAh => "uAh",
            Self::Counts => "counts"
        }
    }
}

pub(crate) trait NxDataset {
    const NAME: &'static str;
    const UNITS: Option<Units> = None;

    fn get_value(&self) -> Self::Type::Target;
}

trait NxDatasetType : Deref {
    type Type : Deref;

    fn get_value(&self) -> Self::Type::Target;
}

pub(crate) struct NexusDatasetData<T> {
    value: T,
}

impl NxDatasetType for NexusDatasetData<String> {
    type Type = String;

    fn get_value(&self) -> Self::Type::Target {
        &self.value
    }
}

impl NxDatasetType for NexusDatasetData<i32> {
    type Type = String;

    fn get_value(&self) -> Self::Type::Target {
        &self.value
    }
}

impl NxDatasetType for NexusDatasetData<f32> {
    type Type = String;

    fn get_value(&self) -> Self::Type::Target {
        &self.value
    }
}

impl NxDatasetType for NexusDatasetData<DateTime<Utc>> {
    type Type = DateTime<Utc>;

    fn get_value(&self) -> Self::Type::Target {
        &self.value
    }
}

pub(crate) mod nx_dataset {
    pub(crate) struct IdfVersion;

    impl NexusDataset<NexusDatasetData<i32>> for IdfVersion {
        
    }
}