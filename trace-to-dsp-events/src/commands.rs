use std::{
    fmt::Display,
    env,
    fs::File,
    io::Write
};

use trace_to_dsp_events::{
    log_then_panic_t,
    trace_iterators::load_from_trace_file::load_trace_file,
};

use crate::{
    SimulationParameters,
    FileParameters,
    DetectionType,
    trace_run::{
        TraceRun,
        BasicParameters,
        AdvancedParameters,
    }
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
pub fn run_simulated_mode(params: SimulationParameters, detection_type : Option<DetectionType>, benchmark : bool, evaluate : bool) {
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

pub fn run_file_mode(params: FileParameters, detection_type : Option<DetectionType>, benchmark : bool, evaluate : bool) {
    let file_name = params.file_name.unwrap_or(
        //"../../Data/Traces/MuSR_A27_B28_C29_D30_Apr2021_Ag_ZF_InstDeg_Slit60_short.traces".to_owned(),
        "../../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces".to_owned(),
    );
    let save_file_name = params.save_file_name.unwrap_or("Saves/output".to_owned());

    let mut trace_file = load_trace_file(&file_name).unwrap();
    let event_index = 243;
    let channel_index = 0;
    let run = trace_file.get_event(event_index).unwrap();
    run_trace(run.normalized_channel_trace(channel_index), save_file_name, detection_type, benchmark, evaluate);
}

fn run_trace(trace: &Vec<u16>, save_file_name: String, detection_type : Option<DetectionType>, benchmark : bool, evaluate : bool) {
    let mut trace_run = TraceRun::new (
        BasicParameters{
            gate_size: 2.,
            smoothing_window_size: 4,
            baseline_length: 1000,
        },
        AdvancedParameters {
            change_detector_threshold: 1.,
            change_detector_bound: 4.,
        },
    );
    let (baselined, smoothed) = trace_run.baselined_from_trace(trace);

    let det_type = detection_type.unwrap_or(DetectionType::Basic);
    let name = match det_type {
        DetectionType::Basic => "Basic Mode",
        DetectionType::Advanced => "Advanced Mode",
    };
    let pulse_events = match det_type {
        DetectionType::Basic => trace_run.run_basic_detection(smoothed.clone()),
        DetectionType::Advanced => trace_run.run_advanced_detection(smoothed.clone()),
    };
    if evaluate {
        trace_run.run_evaluation(name, baselined.clone(), &pulse_events);
    }
    if benchmark {
        trace_run.run_benchmark(smoothed.clone());
        trace_run.save_baselined        (save_file_name.clone(), baselined.clone());
        trace_run.save_smoothed         (save_file_name.clone(), smoothed.clone());
        trace_run.save_pulse_simulation (save_file_name.clone(), baselined.clone(), &pulse_events);
        trace_run.save_pulse_events     (save_file_name, pulse_events);
    }
}
