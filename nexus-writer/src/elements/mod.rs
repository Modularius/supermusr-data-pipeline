pub(crate) mod attribute;
pub(crate) mod dataholder_class;
pub(crate) mod dataset;
pub(crate) mod group;
pub(crate) mod log_value;
pub(crate) mod traits;

#[derive(strum::Display)]
pub(crate) enum NexusUnits {
    #[strum(to_string = "Hz")]
    Hertz,
    #[strum(to_string = "second")]
    Seconds,
    #[strum(to_string = "ms")]
    Milliseconds,
    #[strum(to_string = "us")]
    Microseconds,
    #[strum(to_string = "ns")]
    Nanoseconds,
    #[strum(to_string = "ISO8601")]
    ISO8601,
    #[strum(to_string = "eV")]
    ElectronVolts,
    #[strum(to_string = "MeV")]
    MegaElectronVolts,
    #[strum(to_string = "MeVc^-1")]
    MegaElectronVoltsOverC,
    #[strum(to_string = "uA")]
    MicroAmps,
    #[strum(to_string = "uAh")]
    MicroAmpHours,
    #[strum(to_string = "counts")]
    Counts,
    #[strum(to_string = "mm")]
    Millimeters,
    #[strum(to_string = "mg")]
    Milligrams,
    #[strum(to_string = "mgcm^-3")]
    MilligramsPerCm3,
    #[strum(to_string = "K")]
    Kelvin,
    #[strum(to_string = "G")]
    Gauss,
}
