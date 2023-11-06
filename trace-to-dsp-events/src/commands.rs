use std::{default, env, fmt::Display, fs::File, io::Write};

use anyhow::Result;
use common::Intensity;
use itertools::Itertools;
use rand::{random, seq::IteratorRandom, thread_rng};
use rdkafka::producer::FutureProducer;
use trace_reader::{dispatch_trace_file, load_trace_file};
use trace_to_pulses::{
    basic_muon_detector::{self, BasicMuonAssembler, BasicMuonDetector},
    detectors::threshold_detector::{ThresholdAssembler, ThresholdDetector},
    events::{
        iter::{AssembleFilter, AssemblerIter},
        SaveEventsToFile, SavePulsesToFile,
    },
    log_then_panic_t,
    peak_detector::LocalExtremumDetector,
    processing::{self, make_enumerate_real},
    pulse::Pulse,
    trace_iterators::{find_baseline::FindBaselineFilter, save_to_file::SaveToFile},
    tracedata,
    window::{finite_differences::FiniteDifferences, gate::Gate, WindowFilter},
    EventFilter, Real, RealArray, SmoothingWindow,
};

use crate::{
    trace_run::{AdvancedParameters, BasicParameters}, //, min_max_run::optimize
    FileParameters,
    SimulationParameters,
};

pub async fn run_simulated_mode(params: SimulationParameters) -> Result<()> {
    Ok(())
}

pub async fn run_file_mode(
    params: FileParameters,
    producer: &FutureProducer,
    topic: &str,
) -> Result<()> {
    let file_name = params.file_name.unwrap_or(
        //"../../Data/Traces/MuSR_A27_B28_C29_D30_Apr2021_Ag_ZF_InstDeg_Slit60_short.traces".to_owned(),
        "../../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces"
            .to_owned(),
    );

    let mut trace_file = load_trace_file(&file_name).unwrap();
    let max_event_index = trace_file.get_num_event();
    let num_events = params.num_events.unwrap_or(max_event_index);

    let events = if params.randomize_events {
        let mut rng = thread_rng();
        (0..max_event_index).choose_multiple(&mut rng, num_events)
    } else {
        (0..num_events).collect_vec()
    };

    dispatch_trace_file(trace_file, events, producer, topic, 100).await?;
    Ok(())
}
/*
pub fn run_file_mode(
    params: FileParameters,
) -> Vec<Vec<Real>> {
    let file_name = params.file_name.unwrap_or(
        //"../../Data/Traces/MuSR_A27_B28_C29_D30_Apr2021_Ag_ZF_InstDeg_Slit60_short.traces".to_owned(),
        "../../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces"
            .to_owned(),
    );

    let mut trace_file = load_trace_file(&file_name).unwrap();
    {
        let max_event_index = trace_file.get_num_event();
        let num_events = params.num_events.unwrap_or(max_event_index);

        let events = if params.randomize_events {
            let mut rng = thread_rng();
            (0..max_event_index).choose_multiple(&mut rng, num_events)
        } else {
            (0..num_events).collect_vec()
        };

        events
            .into_iter()
            .map(|event_index| {
                let run = trace_file.get_event(event_index).unwrap();
                (0..trace_file.get_num_channels())
                    .map(|channel_index|run.clone_channel_trace(channel_index))
                    .collect_vec()
            })
            .flatten()
            .collect()
    }
} */

pub fn calc_stats(pulse_vec: &[Pulse]) {
    let num = pulse_vec.len();
    let amplitude_min = {
        let my_iter = pulse_vec
            .iter()
            .map(|pulse| (pulse.peak.value.unwrap_or_default() * 100000000.0) as i32);
        (my_iter.clone().min().unwrap_or_default() as Real) / 100000000.0
    };
    let amplitude_max = {
        let my_iter = pulse_vec
            .iter()
            .map(|pulse| (pulse.peak.value.unwrap_or_default() * 100000000.0) as i32);
        (my_iter.clone().max().unwrap_or_default() as Real) / 100000000.0
    };
    let amplitude_mean: Real = pulse_vec
        .iter()
        .map(|pulse| pulse.peak.value.unwrap_or_default())
        .sum::<Real>()
        / num as Real;
    println!("There are {num} pulses.");
    println!("Whose amplitude ranges from {amplitude_min} to {amplitude_max}, with mean {amplitude_mean}.");
}
