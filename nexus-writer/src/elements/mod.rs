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
    // Time
    #[strum(to_string = "second")]
    Seconds,
    #[strum(to_string = "ms")]
    Milliseconds,
    #[strum(to_string = "ns")]
    Nanoseconds,
    // Energy
    #[strum(to_string = "eV")]
    ElectronVolts,
    #[strum(to_string = "MeV")]
    MegaElectronVolts,
    // Momentum
    #[strum(to_string = "MeVc^-1")]
    MegaElectronVoltsOverC,
    // Current
    #[strum(to_string = "uA")]
    MicroAmps,
    // Charge
    #[strum(to_string = "uAh")]
    MicroAmpHours,
    // Length
    #[strum(to_string = "mm")]
    Millimeters,
    // Mass
    #[strum(to_string = "mg")]
    Milligrams,
    // Density
    #[strum(to_string = "mgcm^-3")]
    MilligramsPerCm3,
    // Temperature
    #[strum(to_string = "K")]
    Kelvin,
    // Magnetic Field
    #[strum(to_string = "G")]
    Gauss,
}
