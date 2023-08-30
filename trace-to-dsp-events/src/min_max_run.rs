use std::iter::once;

use itertools::Itertools;
use num::Integer;
use rand::random;
use trace_to_pulses::{trace_iterators::load_from_trace_file::TraceFile, Real};

use crate::DetectionType;
use crate::trace_run::TraceRun;


#[derive(Clone)]
pub struct TraceResult {
    pub(crate) trace_run : TraceRun,
    pub(crate) highest_score : Real,
    pub(crate) lowest_score : Real,
}
pub struct MinMaxExperiment {
    trace_file : TraceFile,
    detection_type : DetectionType,
    num_events : usize,
    num_channels : usize,
}

impl MinMaxExperiment {
    pub fn new(trace_file : TraceFile, detection_type : DetectionType) -> Self {
        let num_events = trace_file.get_num_event();
        let num_channels = trace_file.get_num_channels();
        Self{
            trace_file,
            detection_type,
            num_events,
            num_channels,
        }
    }

    const NUM_PARAMS_TO_TRY : usize = 35;
    fn run_min_max(&mut self, result: TraceResult, scale : Real) -> TraceResult
    {
        let trace_run = result.trace_run.clone();
        (0..Self::NUM_PARAMS_TO_TRY)
            .map(|i|self.run_max(&trace_run, scale))
            .chain(once(result))
            .min_by_key(|result|(1_000_000_000_000.0*result.highest_score) as i64)
            .unwrap()
    }

    const NUM_EVENTS_TO_TEST : usize = 25;
    fn run_max(&mut self, parent_run : &TraceRun, scale : Real) -> TraceResult {
        let trace_run = TraceRun::mutate_from(parent_run, scale.powi(2));
        let scores = (0..Self::NUM_EVENTS_TO_TEST)
            .map(|_|{
                let run = self.trace_file.get_event(random::<usize>() % self.num_events).unwrap();
                self.run_trace_eval(&trace_run, run.normalized_channel_trace(random::<usize>() % self.num_channels))
            })
            .collect_vec();
        
        let highest_score = scores.iter().max_by_key(|&score|(1_000_000_000_000.0*score) as i64);
        let lowest_score = scores.iter().min_by_key(|&score|(1_000_000_000_000.0*score) as i64);
        TraceResult{
            trace_run,
            highest_score: *highest_score.unwrap_or(&Real::MAX),
            lowest_score: *lowest_score.unwrap_or(&Real::MIN),
        }
    }

    fn run_trace_eval(&self, trace_run : &TraceRun, trace: &Vec<u16>) -> Real {
        let name = match self.detection_type {
            DetectionType::Basic => "Basic Mode",
            DetectionType::Advanced => "Advanced Mode",
        };
        let baselined = trace_run.run_baselined(trace);
        let pulse_events = match self.detection_type {
            DetectionType::Basic => trace_run.run_basic_detection(baselined.clone()),
            DetectionType::Advanced => trace_run.run_advanced_detection(baselined.clone()),
        };
        let (v,d) = trace_run.run_evaluation(name, baselined, &pulse_events);
        d/v
    }
}

const NUM_STEPS : usize = 8;
pub fn optimize(trace_file : TraceFile, detection_type : Option<DetectionType>, repeat : usize) {
    let mut min_max = MinMaxExperiment::new(trace_file,detection_type.unwrap_or(DetectionType::Advanced));
    for n in 0..repeat {
        let mut result = TraceResult{ 
            trace_run: TraceRun::from_random(),
            highest_score: Real::MAX,
            lowest_score: Real::MAX,
        };
        for i in 0..NUM_STEPS {
            result = min_max.run_min_max(result, 1. - 0.35*(i as Real/NUM_STEPS as Real).sqrt());
            println!("Step {i}, Score ({0},{1})",result.lowest_score, result.highest_score);
        }
        println!("{n}: Optimal Result");
        println!("{0:#?}", result.trace_run);
    }
}