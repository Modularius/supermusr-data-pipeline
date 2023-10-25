use std::net::SocketAddr;

use clap::{Parser, Subcommand, ValueEnum};
use common::Intensity;

#[derive(Parser)]
#[clap(author, version, about)]
pub(crate) struct Cli {
    #[clap(long, env)]
    pub(super) broker: String,

    #[clap(long, env)]
    pub(super) username: Option<String>,

    #[clap(long, env)]
    pub(super) password: Option<String>,

    #[clap(long = "group", env)]
    pub(super) consumer_group: String,

    #[clap(long, env)]
    pub(super) trace_topic: String,

    #[clap(long, env)]
    pub(super) event_topic: String,

    #[clap(long, default_value = "127.0.0.1:9090")]
    pub(super) observability_address: SocketAddr,

    #[clap(
        long,
        short = 'd',
        help = "Basic: Finds time/intensitites of events, Advanced: Finds time/intensities/widths and applies feedback corrections"
    )]
    pub detection_type: Option<DetectionType>,

    #[command(subcommand)]
    pub mode: Option<OfflineMode>,
}

#[derive(ValueEnum, Clone)]
pub enum DetectionType {
    Basic,
    Advanced,
}

#[derive(Subcommand, Clone)]
pub enum Mode {
    #[clap(about = "Listen to Kafka Broker and process messages.")]
    Listen,
    #[clap(about = "Read Database Traces and Extract Pulses")]
    Offline(OfflineParameters),
}

#[derive(Parser, Clone)]
pub struct OfflineParameters {
    #[clap(long, short = 'o')]
    pub save_file_name: Option<String>,

    #[command(subcommand)]
    pub mode: Option<OfflineMode>,
}


#[derive(Subcommand, Clone)]
pub enum OfflineMode {
    #[clap(about = "Generate Random Traces and Extract Pulses")]
    Simulation(SimulationParameters),
    #[clap(about = "Read Traces from a File and Extract Pulses")]
    File(FileParameters),
    #[clap(about = "Read Database Traces and Extract Pulses")]
    Database(DatabaseParameters),
}

#[derive(Parser, Clone)]
pub struct SimulationParameters {
    #[clap(long, short = 'l', default_value = "500")]
    pub trace_length: usize,

    #[clap(long, short = 'p', default_value = "3")]
    pub min_pulses: usize,

    #[clap(long, short = 'P', default_value = "10")]
    pub max_pulses: usize,

    #[clap(long, short = 'v', default_value = "0")]
    pub min_voltage: Intensity,

    #[clap(long, short = 'b', default_value = "50")]
    pub base_voltage: Intensity,

    #[clap(long, short = 'V', default_value = "10000")]
    pub max_voltage: Intensity,

    #[clap(long, short = 'n', default_value = "80")]
    pub voltage_noise: Intensity,

    #[clap(long, short = 'd', default_value = "2")]
    pub decay_factor: f64,

    #[clap(long, short = 's', default_value = "2")]
    pub std_dev_min: f64,

    #[clap(long, short = 'S', default_value = "10")]
    pub std_dev_max: f64,

    #[clap(long, short = 't', default_value = "3.0")]
    pub time_wobble: f64,

    #[clap(long, short = 'w', default_value = "0.001")]
    pub value_wobble: f64,

    #[clap(long, short = 'm', default_value = "200")]
    pub min_peak: Intensity,

    #[clap(long, short = 'M', default_value = "900")]
    pub max_peak: Intensity,
}
#[derive(Parser, Clone)]
pub struct FileParameters {
    #[clap(long, short = 'f')]
    pub file_name: Option<String>,

    #[clap(long, short = 'n')]
    pub num_events: Option<usize>,

    #[clap(long, short = 'r', default_value = "false")]
    pub randomize_events: bool,
}

#[derive(Parser, Clone)]
pub struct DatabaseParameters {}
