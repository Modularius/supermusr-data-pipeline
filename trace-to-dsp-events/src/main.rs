//! This crate uses the benchmarking tool for testing the performance of implementated time-series databases.
//!
#![allow(dead_code, unused_variables, unused_imports)]
#![allow(warnings)]
#![warn(missing_docs)]

use common::Intensity;

use anyhow::Result;

use clap::{Parser, Subcommand, ValueEnum};
use trace_to_pulses::{pulse::Pulse, events::SavePulsesToFile};
//use tdengine::utils::log_then_panic_t;

mod commands;
mod trace_run;
//mod min_max_run;

//use tdengine::tdengine::TDEngine;

use crate::commands::{run_file_mode, run_simulated_mode, run_trace, calc_stats, run_simple_detection};

#[derive(Parser)]
#[clap(author, version, about)]
pub(crate) struct Cli {
    #[clap(long, short = 'b', default_value = "true")]
    benchmark: bool,

    #[clap(long, short = 'e', default_value = "true")]
    evaluate: bool,

    #[clap(long, short = 'o')]
    save_file_name: Option<String>,

    #[clap(
        long,
        short = 'd',
        help = "Basic: Finds time/intensitites of events, Advanced: Finds time/intensities/widths and applies feedback corrections"
    )]
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

    #[clap(long, short = 'n')]
    num_events: Option<usize>,
}

#[derive(Parser, Clone)]
struct DatabaseParameters {}

//#[tokio::main]
fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::debug!("Parsing Cli");
    let cli = Cli::parse();

    let traces = match cli.mode {
        Some(Mode::Simulation(npm)) => {
            run_simulated_mode(npm)
        }
        Some(Mode::Database(dpm)) => /* TODO */Vec::<_>::default(),
        Some(Mode::File(fpm)) => {
            run_file_mode(fpm)
        }
        None => run_file_mode(
            FileParameters::parse(),
        ),
    };
    let save_file_name = cli.save_file_name.unwrap_or("Saves/output".to_owned());   //This will be replaced with optional behaviour

    let mut all_pulses = Vec::<Pulse>::new();
    let mut all_pulses_simple = Vec::<Pulse>::new();
    for (index,trace) in traces.into_iter().enumerate() {
        println!("Trace {index}");
        let pulses = run_trace(
            &trace,
            None,
            //Some(save_file_name.clone() + &index.to_string()),
            cli.detection_type.clone(),
            cli.benchmark,
            cli.evaluate,
        );
        all_pulses.extend(pulses.into_iter());
        let pulses = run_simple_detection(&trace, None);
        all_pulses_simple.extend(pulses.into_iter());
    }
    all_pulses.into_iter().save_to_file(&(save_file_name.clone() + "_all_pulses" + ".csv"));
    all_pulses_simple.into_iter().save_to_file(&(save_file_name.clone() + "_all_pulses_simple" + ".csv"));

    Ok(())
}
