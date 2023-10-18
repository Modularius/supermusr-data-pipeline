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

pub fn run_trace(
    trace: &[Real],
    save_file_name: Option<String>,
    detection_type: Option<DetectionType>,
    benchmark: bool,
    evaluate: bool,
) -> Vec<Pulse> {
    
    let basic_parameters = BasicParameters {
        gate_size: 2.,
        min_voltage: 2.,
        smoothing_window_size: 4,
        baseline_length: 1000,
    };
    let advanced_parameters = AdvancedParameters {
        change_detector_threshold: 1.,
        change_detector_bound: 100.,
    };
    
    let baselined = trace
        .iter()
        .enumerate()
        //.map(|(i, v)|(i as Real, *v as Real))
        .map(trace_to_pulses::processing::make_enumerate_real)
        .map(|(i, v)| (i, -v)) // The trace should be positive
        //.find_baseline(basic_parameters.baseline_length) // We find the baseline of the trace from the first 1000 points, which are discarded.
    ;
    let smoothed = baselined
        .clone()
        //.window(Gate::new(basic_parameters.gate_size))                              //  We apply the gate filter first
        //.map(|(i,v)|(i,Real::max(v, basic_parameters.min_voltage)))
        .window(SmoothingWindow::new(basic_parameters.smoothing_window_size))       //  We smooth the trace
        .map(tracedata::extract::enumerated_mean)
    ;
    let events = smoothed
        .clone()
        .window(FiniteDifferences::<2>::new())
        .events(BasicMuonDetector::new(0.00001, 0.0, -0.00001, 0.0, -0.000005, 0.0));
    let pulses = events
        .clone()
        .assemble(BasicMuonAssembler::default())
        //.filter(|pulse|pulse.peak.value.unwrap_or_default() - Real::min(pulse.start.value.unwrap_or_default(),pulse.end.value.unwrap_or_default()) > 0.0001)
        ;

    let pulse_vec = pulses.clone().collect_vec();
    /*

    let (smoothed, feedback_parameter) = trace_run.run_smoothed(baselined.clone());
    let pulses = trace_run.run_pulses(smoothed.clone(), feedback_parameter);
 */
    if evaluate {
        //trace_run.run_and_print_evaluation("General", baselined.clone(), &pulses);
    }
    if benchmark {
        //trace_run.run_benchmark(baselined.clone());
    }
    if let Some(save_file_name) = save_file_name {
        trace
            .iter()
            .enumerate()
            .map(|(i,v)|(i as Real, *v as Real))
            .save_to_file(&(save_file_name.clone() + "_raw.csv"));
        
        baselined.save_to_file(&(save_file_name.clone() + "_baselined" + ".csv"));
        smoothed.save_to_file(&(save_file_name.clone() + "_smoothed" + ".csv"));
        events.save_to_file(&(save_file_name.clone() + "_events" + ".csv"));
        pulses.save_to_file(&(save_file_name.clone() + "_pulses" + ".csv"));
        //time_hist.save_to_file(&(save_file_name.clone() + "_time_hist" + ".csv"));
    }
    pulse_vec
}


pub fn run_simple_detection(
    trace: &[Real],
    save_file_name: Option<String>,
) -> Vec<Pulse> {
    /*
    let basic_parameters = BasicParameters {
        gate_size: 2.,
        min_voltage: 2.,
        smoothing_window_size: 4,
        baseline_length: 1000,
    };
    let advanced_parameters = AdvancedParameters {
        change_detector_threshold: 1.,
        change_detector_bound: 100.,
    };
     */
    let baselined = trace
        .iter()
        .enumerate()
        .map(trace_to_pulses::processing::make_enumerate_real)
        .map(|(i, v)| (i, -v)) // The trace should be positive
    ;
    let events = baselined
        .clone()
        .events(ThresholdDetector::new(0.39965, 4));
    let pulses = events
        .clone()
        .assemble(ThresholdAssembler::default())
        ;

    let pulse_vec = pulses.clone().collect_vec();
    if let Some(save_file_name) = save_file_name {
        trace
            .iter()
            .enumerate()
            .map(|(i,v)|(i as Real, *v as Real))
            .save_to_file(&(save_file_name.clone() + "_raw.csv"));
        pulses.save_to_file(&(save_file_name.clone() + "_pulses" + ".csv"));
    }
    pulse_vec
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