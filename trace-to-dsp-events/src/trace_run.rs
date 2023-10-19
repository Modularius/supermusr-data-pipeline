use std::{collections::VecDeque, fmt::Debug, rc::Rc, time::Instant};

use anyhow::{anyhow, Result};
use common::Intensity;
use itertools::Itertools;

use rand::{random, seq::SliceRandom};
use trace_to_pulses::{
    change_detector::{ChangeClass, ChangeDetector, ChangeEvent},
    detectors::{muon_detector::MuonDetector, threshold_detector::{ThresholdDetector, ThresholdAssembler, ThresholdDuration}},
    events::{SaveEventsToFile, iter::AssembleFilter, SavePulsesToFile},
    peak_detector::{self, LocalExtremumDetector},
    processing,
    pulse_detector::{Biexponential, Gaussian, PulseDetector},
    trace_iterators::{
        feedback::{FeedbackFilter, FeedbackParameter as FP},
        find_baseline::FindBaselineFilter,
        save_to_file::SaveToFile,
        to_trace::ToTrace,
    },
    tracedata::{self, Stats, TraceData},
    window::{
        composite::{CompositeWindow, DoublingWindow},
        exponential_smoothing_window::ExponentialSmoothingWindow,
        finite_differences::{self, FiniteDifferences},
        gate::Gate,
        trivial::TrivialWindow,
        WindowFilter,
    },
    EventFilter, EventsWithFeedbackFilter, Real, RealArray, SmoothingWindow, TraceArray, TracePair, pulse::Pulse, basic_muon_detector::{BasicMuonAssembler, BasicMuonDetector},
};


pub fn save_raw_file(trace: &[Real], save_file_name: Option<&str>) {
    if let Some(save_file_name) = save_file_name {
        trace
            .iter()
            .enumerate()
            .map(|(i,v)|(i as Real, *v as Real))
            .save_to_file(&(save_file_name.to_owned() + "_raw.csv"));
    }
}


#[derive(Default, Debug, Clone)]
pub struct BasicParameters {
    pub gate_size: Real,
    pub min_voltage: Real,
    pub smoothing_window_size: usize,
    pub baseline_length: usize,
    pub min_amplitude: Real,
    pub muon_onset: ThresholdDuration,
    pub muon_fall: ThresholdDuration,
    pub muon_termination: ThresholdDuration,
}

pub fn run_basic_detection(
    trace: &[Real],
    parameters : &BasicParameters,
    save_file_name: Option<&str>,
) -> Vec<Pulse> {
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
        .window(SmoothingWindow::new(parameters.smoothing_window_size))       //  We smooth the trace
        .map(tracedata::extract::enumerated_mean)
    ;
    let events = smoothed
        .clone()
        .window(FiniteDifferences::<2>::new())
        .events(BasicMuonDetector::new(
            &parameters.muon_onset,
            &parameters.muon_fall,
            &parameters.muon_termination
        ));
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
    if let Some(save_file_name) = save_file_name {
        
        baselined.save_to_file(&(save_file_name.to_owned() + "_baselined" + ".csv"));
        smoothed.save_to_file(&(save_file_name.to_owned() + "_smoothed" + ".csv"));
        events.save_to_file(&(save_file_name.to_owned() + "_events" + ".csv"));
        pulses.save_to_file(&(save_file_name.to_owned() + "_pulses" + ".csv"));
        //time_hist.save_to_file(&(save_file_name.clone() + "_time_hist" + ".csv"));
    }
    pulse_vec
}





#[derive(Default, Debug, Clone)]
pub struct SimpleParameters {
    pub gate_size: Real,
    pub min_voltage: Real,
    pub smoothing_window_size: usize,
    pub baseline_length: usize,
    pub threshold_trigger: ThresholdDuration,
}

pub fn run_simple_detection(
    trace: &[Real],
    parameters : &SimpleParameters,
    save_file_name: Option<&str>,
) -> Vec<Pulse> {
    let baselined = trace
        .iter()
        .enumerate()
        .map(trace_to_pulses::processing::make_enumerate_real)
        .map(|(i, v)| (i, -v)) // The trace should be positive
    ;
    let events = baselined
        .clone()
        .events(ThresholdDetector::new(&parameters.threshold_trigger));
    let pulses = events
        .clone()
        .assemble(ThresholdAssembler::default())
        ;

    let pulse_vec = pulses.clone().collect_vec();
    if let Some(save_file_name) = save_file_name {
        pulses.save_to_file(&(save_file_name.to_owned() + "_pulses" + ".csv"));
    }
    pulse_vec
}

#[derive(Default, Debug, Clone)]
pub(crate) struct AdvancedParameters {
    pub(crate) change_detector_threshold: Real,
    pub(crate) change_detector_bound: Real,
}
