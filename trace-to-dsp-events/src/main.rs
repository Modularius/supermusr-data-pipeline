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
    ChangeDetector,
    ChangeEvent,
};
use trace_to_dsp_events::detectors::{
    composite::CompositeTopOnlyDetector,
    pulse_detector::PulseDetector,
    peak_detector::{PeakClass, PeakDetector},
};
use trace_to_dsp_events::tagged::{Stats, extract};
use trace_to_dsp_events::partition::PartitionFilter;

use trace_to_dsp_events::window::noise_smoothing_window::NoiseSmoothingWindow;
use trace_to_dsp_events::{
    events::{
        EventFilter,
        Event,
        SaveEventsToFile,
    },
    processing,
    trace_iterators::{
        load_from_trace_file::load_trace_file,
        save_to_file::SaveToFile,
        find_baseline::FindBaselineFilter,
        finite_difference,
        finite_difference::{
            FiniteDifferencesFilter,
            FiniteDifferencesIter,
        },
        to_trace::ToTrace,
        feedback::FeedbackFilter,
    },
    window::{
        WindowFilter,
        gate::Gate,
        smoothing_window::SmoothingWindow,
    },
    Real,
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
    #[clap(long, short = 'o')]
    save_file_name: Option<String>,
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
    let save_file_name = params.save_file_name.unwrap_or("Saves/output".to_owned());

    let mut trace_file = load_trace_file(&file_name).unwrap();
    let event_index = 243;
    let channel_index = 0;
    let run = trace_file.get_event(event_index).unwrap();
    run_trace(run.normalized_channel_trace(channel_index), save_file_name);
}

fn run_trace(trace: &Vec<u16>, save_file_name: String) {
    let trace_baselined = trace
        .iter()
        .enumerate()
        .map(processing::make_enumerate_real)
        .map(|(i,v)| (i,-v))
        .find_baseline(1000)
        .collect_vec();

    let trace_smoothed = trace_baselined
        .iter()
        .cloned()
        .window(Gate::new(2.))
        .feedback(|x,y|x + y,0.)
        .window(SmoothingWindow::new(4))
        .collect_vec();

    let pulse_events = trace_smoothed
        .iter()
        .cloned()
        .events(PulseDetector::new(PeakDetector::new()))
        .collect_vec();
/*
    let mut pulse_events = trace_smoothed
        .clone()
        .into_iter()
        .partition_on_detections(PeakDetector::new())
        //.filter(|trace_partition|trace_partition.get_event().get_data().get_class() == PeakClass::LocalMax)
        .map(|trace_partition| {
            let event = trace_partition.get_event();
            let a = event.get_data().get_value().unwrap_or_default();
            let mu = event.get_time();
            PulseEvent {
                time: mu,
                data: PulseData::new(
                    Some(mu),
                    Some(a),
                    trace_partition.iter()
                        .find(|(_,v)| *v > a*(Real::exp(-0.5)))
                        .map(|(i,_)|mu - i)
                )
            }
        })
        .collect_vec();
    for i in 0..pulse_events.len() {
        let time = pulse_events[i].get_time();
        //pulse_events[i].get_data_mut().set_peak_time(Some(trace_baselined[time as usize - 999].1));
        let gaussian = Gaussian::new(
            pulse_events[i].get_data().get_peak_intensity().unwrap_or_default(),
            time,
            pulse_events[i].get_data().get_radius().unwrap_or_default(),
        );
        pulse_events[(i+1)..].iter_mut().for_each(|event| {
            let new_intensity = event.get_data().get_peak_intensity().map(|v|v - gaussian.get_value_at(event.get_time()));
            event.get_data_mut().set_peak_intensity(new_intensity);
        });
    } */
    let trace_simulated = trace_baselined
        .iter()
        .cloned()
        .to_trace(pulse_events.as_slice());

    let (v,d) = trace_baselined
        .iter()
        .cloned()
        .evaluate_events(&pulse_events)
        .map(|(_,v,d)|(v*v,d*d))
        .fold((0.,0.),|(full_v,full_d), (v,d)|(full_v + v, full_d + d));
    println!("Ratio of Diff = {0}/{1} = {2}", Real::sqrt(d), Real::sqrt(v), Real::sqrt(d/v));

    trace_baselined
        .iter()
        .cloned()
        .save_to_file(&(save_file_name.clone() + "_baselined" + ".csv")).unwrap();
    trace_smoothed
        .iter()
        .cloned()
        .map(extract::enumerated_mean)
        .save_to_file(&(save_file_name.clone() + "_smoothed" + ".csv")).unwrap();
    //SaveToFile::save_to_file(trace_simulated.into_iter(), &(save_file_name.clone() + "_simulated" + ".csv")).unwrap();
    SaveEventsToFile::save_to_file(pulse_events.into_iter(), &(save_file_name.clone() + "_pulse" + ".csv")).unwrap();
    //pulse_events.save_to_file(&(save_file_name.clone() + "5" + ".csv")).unwrap();
}
