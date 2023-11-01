//! This crate uses the benchmarking tool for testing the performance of implementated time-series databases.
//!
#![allow(dead_code, unused_variables, unused_imports)]
#![allow(warnings)]
#![warn(missing_docs)]

use clap::Parser;

use common::Intensity;

use anyhow::Result;

use rdkafka::consumer::{StreamConsumer, Consumer};
use rdkafka::producer::FutureProducer;
use trace_to_pulses::Real;
use trace_to_pulses::detectors::threshold_detector::ThresholdDuration;
use trace_to_pulses::trace_iterators::save_to_file::SaveToFile;
use trace_to_pulses::{pulse::Pulse, events::SavePulsesToFile};
//use tdengine::utils::log_then_panic_t;

mod commands;
mod trace_run;
mod clap_structs;
mod listen;

use clap_structs::{Cli, Mode, FileParameters, SimulationParameters, DatabaseParameters, OfflineParameters, OfflineMode};

use crate::commands::{
    run_file_mode, run_simulated_mode, calc_stats
};
use crate::trace_run::{
    run_basic_detection, run_simple_detection, BasicParameters, SimpleParameters, save_raw_file
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::debug!("Parsing Cli");
    let cli = Cli::parse();

    let mut client_config = common::generate_kafka_client_config(&cli.broker, &cli.username, &cli.password);

    let producer: FutureProducer = client_config.create()?;
    match cli.mode {
        Some(OfflineMode::Simulation(npm)) => { run_simulated_mode(npm).await? },
        Some(OfflineMode::Database(dpm)) => {},
        Some(OfflineMode::File(fpm)) => { run_file_mode(fpm, &producer, &cli.trace_topic).await? },
        _ => {}
    }

    let consumer: StreamConsumer = client_config
        .set("group.id", &cli.consumer_group)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()?;

    consumer.subscribe(&[&cli.trace_topic])?;
    loop {
        listen::listen(&consumer, &producer, &cli.event_topic).await?;
    }
    Ok(())
}
/*
fn run_offline_mode(opm : OfflineParameters) -> Result<()> {
    
    let save_file_name = opm.save_file_name.unwrap_or("Saves/output".to_owned());   //This will be replaced with optional behaviour

    let mut all_pulses_basic = Vec::<Pulse>::new();
    let mut all_pulses_simple = Vec::<Pulse>::new();
    
    let mut basic_parameters = BasicParameters {
        gate_size: 2.,
        min_voltage: 2.,
        smoothing_window_size: 4,
        baseline_length: 1000,
        max_amplitude: Some(0.4),
        min_amplitude: Some(0.3985),
        muon_onset: ThresholdDuration{threshold: 0.00001, duration: 0},
        muon_fall: ThresholdDuration{threshold: -0.00001, duration: 0},
        muon_termination: ThresholdDuration{threshold: -0.000005, duration: 0},
    };

    let mut simple_parameters = SimpleParameters {
        threshold_trigger: ThresholdDuration { threshold: 0.39875, duration: 4},
        ..Default::default()
    };
    
    for (index,trace) in traces.into_iter().enumerate() {
        println!("Trace {index}");
        //let save_file_name = Some(save_file_name.clone() + &index.to_string());
        let save_file_name = None::<&str>;
        let save_file_name = save_file_name.as_deref();
        append_simple_and_basic_detections(&trace, save_file_name, &simple_parameters, &mut all_pulses_simple, &basic_parameters, &mut all_pulses_basic);

    }
    all_pulses_basic.into_iter().save_to_file(&(save_file_name.clone() + "_all_pulses_simple.csv"));
    all_pulses_simple.into_iter().save_to_file(&(save_file_name.clone() + "_all_pulses_simple.csv"));
    Ok(())
} */

fn perform_simple_and_basic_detections(traces : &[&[Real]],
    simple_parameters : &SimpleParameters, basic_parameters : &BasicParameters
) -> (Vec<Pulse>, Vec<Pulse>) {
    let mut all_pulses_basic = Vec::<Pulse>::new();
    let mut all_pulses_simple = Vec::<Pulse>::new();
    
    for (index,trace) in traces.into_iter().enumerate() {
        println!("Trace {index}");
        //let save_file_name = Some(save_file_name.clone() + &index.to_string());
        let save_file_name = None::<&str>;
        let save_file_name = save_file_name.as_deref();
        append_simple_and_basic_detections(&trace, save_file_name, simple_parameters, &mut all_pulses_simple, basic_parameters, &mut all_pulses_basic);
    }

    (all_pulses_simple, all_pulses_basic)
}

fn append_simple_and_basic_detections(trace : &[Real], save_file_name : Option<&str>,
    simple_parameters : &SimpleParameters, all_pulses_simple: &mut Vec<Pulse>,
    basic_parameters : &BasicParameters, all_pulses_basic: &mut Vec<Pulse>
) {
    save_raw_file(&trace, save_file_name);

    let pulses = run_basic_detection(
        &trace,
        basic_parameters,
        save_file_name
    );
    all_pulses_basic.extend(pulses.into_iter());

    let pulses = run_simple_detection(
        &trace,
        simple_parameters,
        save_file_name
    );
    all_pulses_simple.extend(pulses.into_iter());
}