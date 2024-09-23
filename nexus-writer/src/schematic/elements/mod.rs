use builder::{NexusBuilder};
use hdf5::{
    types::{StringError, TypeDescriptor},
    Dataset, Group, H5Type,
};
use log_value::NumericVector;
use thiserror::Error;

use crate::error::{NexusDatasetError, NexusPushError};

pub(crate) mod attribute;
pub(crate) mod builder;
pub(crate) mod dataset;
pub(crate) mod group;
pub(crate) mod traits;
pub(crate) mod dataholder_class;
pub(crate) mod log_value;

#[derive(strum::Display)]
pub(crate) enum NexusUnits {
    #[strum(to_string = "second")]
    Seconds,
    #[strum(to_string = "us")]
    Microseconds,
    #[strum(to_string = "ns")]
    Nanoseconds,
    #[strum(to_string = "ISO8601")]
    ISO8601,
    #[strum(to_string = "mEv")]
    MegaElectronVolts,
    #[strum(to_string = "uAh")]
    MicroAmpHours,
    #[strum(to_string = "counts")]
    Counts,
}
