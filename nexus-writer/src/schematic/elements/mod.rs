pub(crate) mod attribute;
pub(crate) mod builder;
pub(crate) mod dataholder_class;
pub(crate) mod dataset;
pub(crate) mod group;
pub(crate) mod log_value;
pub(crate) mod traits;

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
