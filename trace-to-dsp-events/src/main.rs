//! This crate uses the benchmarking tool for testing the performance of implementated time-series databases.
//!
#![allow(dead_code, unused_variables, unused_imports)]
#![allow(warnings)]
#![warn(missing_docs)]


use common::Intensity;

use anyhow::Result;

use clap::{Parser, Subcommand, ValueEnum};
//use tdengine::utils::log_then_panic_t;

mod commands;
mod trace_run;

//use tdengine::tdengine::TDEngine;

use crate::commands::{run_file_mode, run_simulated_mode};

#[derive(Parser)]
#[clap(author, version, about)]
pub(crate) struct Cli {
    #[clap(long, short = 'b', default_value = "true")]
    benchmark: bool,

    #[clap(long, short = 'e', default_value = "true")]
    evaluate: bool,

    #[clap(long, short = 'f', help = "Basic: Finds time/intensitites of events, Advanced: Finds time/intensities/widths and applies feedback corrections")]
    detection_type: Option<DetectionType>,
    
    #[command(subcommand)]
    mode: Option<Mode>,
}

#[derive(ValueEnum, Clone)]
pub enum DetectionType {
    Basic,
    Advanced,
}

#[derive(Subcommand, Clone)]
enum Mode {
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
    trace_length: usize,

    #[clap(long, short = 'p', default_value = "3")]
    min_pulses: usize,

    #[clap(long, short = 'P', default_value = "10")]
    max_pulses: usize,

    #[clap(long, short = 'v', default_value = "0")]
    min_voltage: Intensity,

    #[clap(long, short = 'b', default_value = "50")]
    base_voltage: Intensity,

    #[clap(long, short = 'V', default_value = "10000")]
    max_voltage: Intensity,

    #[clap(long, short = 'n', default_value = "80")]
    voltage_noise: Intensity,

    #[clap(long, short = 'd', default_value = "2")]
    decay_factor: f64,

    #[clap(long, short = 's', default_value = "2")]
    std_dev_min: f64,

    #[clap(long, short = 'S', default_value = "10")]
    std_dev_max: f64,

    #[clap(long, short = 't', default_value = "3.0")]
    time_wobble: f64,

    #[clap(long, short = 'w', default_value = "0.001")]
    value_wobble: f64,

    #[clap(long, short = 'm', default_value = "200")]
    min_peak: Intensity,

    #[clap(long, short = 'M', default_value = "900")]
    max_peak: Intensity,
}
#[derive(Parser, Clone)]
pub struct FileParameters {
    #[clap(long, short = 'f')]
    file_name: Option<String>,
    #[clap(long, short = 'o')]
    save_file_name: Option<String>,
}

#[derive(Parser, Clone)]
struct DatabaseParameters {}

//#[tokio::main]
fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::debug!("Parsing Cli");
    let cli = Cli::parse();

    match cli.mode {
        Some(Mode::Simulation(npm)) => run_simulated_mode(npm, cli.detection_type, cli.benchmark, cli.evaluate),
        Some(Mode::Database(dpm)) => (),
        Some(Mode::File(fpm)) => run_file_mode(fpm, cli.detection_type, cli.benchmark, cli.evaluate),
        None => run_file_mode(FileParameters::parse(), cli.detection_type, cli.benchmark, cli.evaluate),
    }

    Ok(())
}
