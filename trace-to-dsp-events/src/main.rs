//! This crate uses the benchmarking tool for testing the performance of implementated time-series databases.
//!
#![allow(dead_code, unused_variables, unused_imports)]
#![warn(missing_docs)]

use std::env;
use std::fmt::Display;
use std::os::unix::process;
use std::{fs::File, io::Write};
use std::{thread, time::Instant};

use common::Intensity;
use common::Time;

use anyhow::Result;

use clap::{Parser, Subcommand};
use dotenv;

use itertools::Itertools;
use tdengine::utils::log_then_panic_t;
use trace_simulator::generator::{PulseDistribution, RandomInterval};

use trace_to_dsp_events::detectors::change_detector::{
    ChangeDetector, SignDetector, SimpleChangeDetector,
};
use trace_to_dsp_events::detectors::composite::CompositeDetector;
use trace_to_dsp_events::events::EventWithData;
use trace_to_dsp_events::trace_iterators::finite_difference::{
    self, FiniteDifferencesFilter, FiniteDifferencesIter,
};
use trace_to_dsp_events::window::{noise_smoothing_window::NoiseSmoothingWindow, WindowFilter};
use trace_to_dsp_events::{
    detectors::{event_detector::EventsDetector, peak_detector::PeakDetector},
    events::Event,
    events::SimpleEvent,
    processing,
    trace_iterators::{load_from_trace_file::load_trace_file, save_to_file::SaveToFile},
    window::{
        composite::CompositeWindow,
        gate::Gate,
        smoothing_window::{self, SmoothingWindow, Stats},
    },
    EventFilter, Integer, Real, TraceMakerFilter,
};

use tdengine::tdengine::TDEngine;
use trace_simulator;

#[derive(Parser)]
#[clap(author, version, about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    mode: Option<Mode>,
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
struct SimulationParameters {
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
struct FileParameters {
    #[clap(long, short = 'f')]
    file_name: Option<String>,
}

#[derive(Parser, Clone)]
struct DatabaseParameters {}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::debug!("Parsing Cli");
    let cli = Cli::parse();

    match cli.mode {
        Some(Mode::Simulation(npm)) => run_simulated_mode(npm),
        Some(Mode::Database(dpm)) => (),
        Some(Mode::File(fpm)) => run_file_mode(fpm),
        None => run_file_mode(FileParameters::parse()),
    }

    Ok(())
}

fn save_to_file<T: Display, I: Iterator<Item = T>>(name: &str, it: I) {
    let cd = env::current_dir()
        .unwrap_or_else(|e| log_then_panic_t(format!("Cannot obtain current directory : {e}")));
    let path = cd.join(name);
    let mut file = File::create(path)
        .unwrap_or_else(|e| log_then_panic_t(format!("Cannot create {name} : {e}")));
    it.for_each(|v| {
        write!(file, "{v},")
            .unwrap_or_else(|e| log_then_panic_t(format!("Cannot write to {name} : {e}")))
    });
    writeln!(&mut file)
        .unwrap_or_else(|e| log_then_panic_t(format!("Cannot event to {name} : {e}")));
}

fn run_simulated_mode(params: SimulationParameters) {
    /*
    let distrbution = PulseDistribution {
        std_dev: RandomInterval(params.std_dev_min,params.std_dev_max),
        decay_factor: RandomInterval(0.,params.decay_factor),
        time_wobble: RandomInterval(0.,params.time_wobble),
        value_wobble: RandomInterval(0.,params.value_wobble),
        peak: RandomInterval(params.min_peak as f64,params.max_peak as f64)
    };

    let pulses = trace_simulator::create_pulses(
        params.trace_length,
        params.min_pulses,
        params.max_pulses,
        &distrbution,
    );
    let trace = trace_simulator::create_trace(
        params.trace_length,
        pulses,
        params.min_voltage,
        params.base_voltage,
        params.max_voltage,
        params.voltage_noise,
    );
    */
}
fn run_file_mode(params: FileParameters) {
    let file_name = params.file_name.unwrap_or(
        //"../../Data/Traces/MuSR_A27_B28_C29_D30_Apr2021_Ag_ZF_InstDeg_Slit60_short.traces".to_owned(),
        "../../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces".to_owned(),
    );
    let mut trace_file = load_trace_file(&file_name).unwrap();
    let event_index = 243;
    let channel_index = 0;
    let run = trace_file.get_event(event_index).unwrap();

    let iter_enumerate = run
        .normalized_channel_trace(channel_index)
        .iter()
        .enumerate();

    let events: Vec<_> = iter_enumerate
        .clone()
        .map(processing::make_enumerate_real)
        .map(|(i,v)| (i,100000. + v))
        .window(Gate::new(2.))
        .window(SmoothingWindow::new(3))
        .map(smoothing_window::extract::enumerated_normalised_value)
        .finite_differences()
        .window(CompositeWindow::new([
            Box::new(SmoothingWindow::new(8)),
            Box::new(SmoothingWindow::new(8)),
            Box::new(SmoothingWindow::new(8)),
        ]))
        .map(|(i, stats)| (i, stats.map(smoothing_window::extract::mean)))
        .events(EventsDetector::new(
            CompositeDetector::new([
                Box::new(SimpleChangeDetector::new(1.)),
                Box::new(SimpleChangeDetector::new(1.)),
                Box::new(SimpleChangeDetector::new(1.)),
        ]),
        SmoothingWindow::new(50)))
        /*.events(FiniteDifferenceChangeDetector::new([
            SimpleChangeDetector::new(1.),
            SimpleChangeDetector::new(1.),
        ]))*/
        //.flat_map(|m| m.into_iter())
        .collect();
    println!("{:?}", events.iter().len());
    let mut counter: i32 = 0;
    for event in events.iter() {
        let index = event.get_time();
        let data = event.get_data();
        println!("{:?}", event);
    }
    //.window(NoiseSmoothingWindow::new(5,0.5,0.))
    //.map(smoothing_window::extract::enumerated_mean)
    //.save_to_file("../../Data/CSV/trace1.csv")
    //.unwrap();
}
