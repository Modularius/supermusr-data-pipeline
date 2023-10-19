use std::{env, fmt::Display, fs::File, io::Write, default};

use histogram;
use common::Intensity;
use itertools::Itertools;
use rand::{random, seq::IteratorRandom, thread_rng};
use trace_to_pulses::{
    log_then_panic_t,
    trace_iterators::{load_from_trace_file::{load_trace_file, TraceFile}, save_to_file::SaveToFile, find_baseline::FindBaselineFilter},
    Real, processing::{self, make_enumerate_real}, window::{WindowFilter, gate::Gate, finite_differences::FiniteDifferences}, SmoothingWindow, peak_detector::LocalExtremumDetector,
    EventFilter, RealArray, tracedata,
    events::{SaveEventsToFile, SavePulsesToFile, iter::{AssembleFilter, AssemblerIter}},
    basic_muon_detector::{self, BasicMuonDetector, BasicMuonAssembler}, pulse::Pulse, detectors::threshold_detector::{ThresholdDetector, ThresholdAssembler},
};

use crate::{
    trace_run::{AdvancedParameters, BasicParameters}, //, min_max_run::optimize
    DetectionType,
    FileParameters,
    SimulationParameters,
};
//use trace_simulator;
/*
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
 */
pub fn run_simulated_mode(
    params: SimulationParameters,
) -> Vec<Vec<Real>> {
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
    Vec::<Vec<_>>::default()
}

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
        let max_event_index = 300;

        let num_events = params.num_events.unwrap_or(max_event_index);
        let mut rng = thread_rng();
        let events = (0..max_event_index).choose_multiple(&mut rng, num_events);

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
}


pub fn calc_stats(pulse_vec : &[Pulse]) {
    let num = pulse_vec.len();
    let amplitude_min = {
        let my_iter = pulse_vec
            .iter()
            .map(|pulse|(pulse.peak.value.unwrap_or_default()*100000000.0) as i32);
            (my_iter.clone().min().unwrap_or_default() as Real)/100000000.0
    };
    let amplitude_max = {
        let my_iter = pulse_vec
            .iter()
            .map(|pulse|(pulse.peak.value.unwrap_or_default()*100000000.0) as i32);
            (my_iter.clone().max().unwrap_or_default() as Real)/100000000.0
    };
    let amplitude_mean : Real = pulse_vec
        .iter()
        .map(|pulse|pulse.peak.value.unwrap_or_default())
        .sum::<Real>()/num as Real;
    println!("There are {num} pulses.");
    println!("Whose amplitude ranges from {amplitude_min} to {amplitude_max}, with mean {amplitude_mean}.");

}