use itertools::Itertools;
use num::Integer;
use rand::random;
use trace_to_pulses::{trace_iterators::load_from_trace_file::TraceFile, Real};

use crate::DetectionType;
use crate::trace_run::{TraceResult, TraceRun};


const NUM_PARAMS_TO_TRY : usize = 10000;
const NUM_EVENTS_TO_TEST : usize = 20;
fn run_min_max_experiment(mut trace_file : TraceFile, detection_type : Option<DetectionType>)
{
    let num_events = trace_file.get_num_event();
    let num_channels = trace_file.get_num_channels();

    let result : Option<TraceResult> = (0..NUM_PARAMS_TO_TRY).map(|i|{
        if i.mod_floor(&100) == 0 {
            println!("{i}");
        }

        let trace_run = TraceRun::from_random();
        let scores = (0..NUM_EVENTS_TO_TEST)
            .map(|_|trace_file.get_event(random::<usize>() % num_events).unwrap())
            .map(|run|
                run_trace_eval(&trace_run, run.normalized_channel_trace(random::<usize>() % num_channels), detection_type.clone())
            )
            .collect_vec();
        
        let highest_score = scores.iter().max_by_key(|&score|(1_000_000_000_000.0*score) as i64);
        let lowest_score = scores.iter().min_by_key(|&score|(1_000_000_000_000.0*score) as i64);
        TraceResult{
            trace_run,
            highest_score: *highest_score.unwrap_or(&Real::MAX),
            lowest_score: *lowest_score.unwrap_or(&Real::MIN),
        }
    }).min_by_key(|result|(1_000_000_000_000.0*result.highest_score) as i64);
    match result {
        Some(TraceResult{ trace_run, highest_score, lowest_score })
            => println!("Found optimal: {trace_run:#?} with score ({lowest_score},{highest_score})."),
        None
            => println!("No optimum found"),
    }

}


fn run_trace_eval(trace_run : &TraceRun, trace: &Vec<u16>, detection_type : Option<DetectionType>) -> Real {
    let (baselined, smoothed) = trace_run.baselined_from_trace(trace);

    let det_type = detection_type.unwrap_or(DetectionType::Advanced);
    let name = match det_type {
        DetectionType::Basic => "Basic Mode",
        DetectionType::Advanced => "Advanced Mode",
    };
    let pulse_events = match det_type {
        DetectionType::Basic => trace_run.run_basic_detection(smoothed.clone()),
        DetectionType::Advanced => trace_run.run_advanced_detection(smoothed.clone()),
    };
    let (v,d) = trace_run.run_evaluation(name, baselined.clone(), &pulse_events);
    d/v
}