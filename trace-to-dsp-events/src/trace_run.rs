use std::{time::Instant, fmt::Debug};

use itertools::Itertools;

use rand::{random, seq::SliceRandom};
use trace_to_pulses::{
    Real,
    trace_iterators::{
        find_baseline::FindBaselineFilter,
        feedback::{FeedbackFilter, EndFeedbackFilter, OptFeedParam},
        to_trace::ToTrace, save_to_file::SaveToFile
    },
    processing,
    window::{
        gate::Gate,
        WindowFilter
    },
    SmoothingWindow,
    tracedata::{self, Stats},
    pulse_detector::{PulseDetector, PulseEvent},
    peak_detector::{self, LocalExtremumDetector},
    change_detector::ChangeDetector,
    EventWithFeedbackFilter,
    EventFilter, events::SaveEventsToFile,
};

fn time_collect_vec<I : Iterator + Clone>(iter : &I) -> (Vec<I::Item>, Real) {
    let timer = Instant::now();
    (iter.clone().collect_vec(), timer.elapsed().as_micros() as Real*0.001)
}

#[derive(Debug)]
pub(crate) struct BasicParameters {
    pub(crate) gate_size : Real,
    pub(crate) smoothing_window_size : usize,
    pub(crate) baseline_length : usize,
}
#[derive(Debug)]
pub(crate) struct AdvancedParameters {
    pub(crate) change_detector_threshold : Real,
    pub(crate) change_detector_bound : Real,
}

#[derive(Debug)]
pub(crate) struct TraceRun
{
    basic_parameters: BasicParameters,
    advanced_parameters: AdvancedParameters,
}
impl TraceRun {
    pub fn new(basic_parameters : BasicParameters, advanced_parameters : AdvancedParameters) -> Self {
        Self { basic_parameters, advanced_parameters }
    }
    pub fn from_random() -> Self {
        TraceRun {
            basic_parameters: BasicParameters{
                gate_size: 2.*random::<Real>() + 1.,
                smoothing_window_size: (3.*random::<Real>() + 3.) as usize,
                baseline_length: (4000.*random::<Real>() + 500.) as usize,
            },
            advanced_parameters: AdvancedParameters {
                change_detector_threshold: 1.*random::<Real>() + 0.5,
                change_detector_bound: 2.*random::<Real>() + 2.5,
            },
        }
    }

    pub fn mutate_from(source : &Self, scale : Real) -> Self {
        TraceRun {
            basic_parameters: BasicParameters{
                gate_size: Real::max(0., source.basic_parameters.gate_size + scale*(2.*random::<Real>() - 1.)),
                smoothing_window_size: i32::max(2, source.basic_parameters.smoothing_window_size as i32 + [1,0,0,0,-1].choose(&mut rand::thread_rng()).unwrap()) as usize,
                baseline_length: Real::max(100.0, source.basic_parameters.baseline_length as Real + scale*1000.*(2.*random::<Real>() - 1.)) as usize,
            },
            advanced_parameters: AdvancedParameters {
                change_detector_threshold: Real::max(0.25, source.advanced_parameters.change_detector_threshold + scale*(2.*random::<Real>() - 1.)),
                change_detector_bound: Real::max(1.5, source.advanced_parameters.change_detector_bound + scale*(2.*random::<Real>() - 1.)),
            },
        }
    }
    pub fn baselined_from_trace<'a>(&self, trace: &'a Vec<u16>) -> (impl Iterator<Item = (Real,Real)> + Clone + 'a, impl Iterator<Item = (Real,Stats,OptFeedParam<Real>)> + Clone + 'a) {
        let baselined = trace.iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .map(|(i,v)| (i,-v))                                        // The trace should be positive
            .find_baseline(self.basic_parameters.baseline_length)            // We find the baseline of the trace from the first 1000 points, which are discarded.
        ;
        let smoothed = baselined
            .clone()
            .window(Gate::new(self.basic_parameters.gate_size))                              //  We apply the gate filter first
            //.start_feedback(tracedata::operation::add_real)                           //  We apply the feedback here
            .window(SmoothingWindow::new(self.basic_parameters.smoothing_window_size))       //  We smooth the trace
            .start_feedback(tracedata::operation::shift_stats)                          
        ;
        (baselined,smoothed)
    }

    pub fn run_basic_detection<'a>(&self, smoothed : impl Iterator<Item = (Real,Stats,OptFeedParam<Real>)>) -> Vec<PulseEvent> {
        smoothed
            .map(tracedata::extract::enumerated_mean)
            .events(LocalExtremumDetector::new())
            .tuple_windows()
            .filter_map(peak_detector::local_extrema_to_peaks)
            .map(PulseEvent::from)
            .collect_vec()
    }

    pub fn run_advanced_detection<'a>(&self, smoothed : impl Iterator<Item = (Real,Stats,OptFeedParam<Real>)>) -> Vec<PulseEvent> {
        smoothed
            //.events_with_feedback(PulseDetector::new(LocalExtremeDetector::new()))
            .events_with_feedback(PulseDetector::new(ChangeDetector::new(self.advanced_parameters.change_detector_threshold), self.advanced_parameters.change_detector_bound))
            .collect_vec()
    }
    pub fn run_evaluation<'a>(&self, name : &str, baselined: impl Iterator<Item = (Real,Real)> + Clone, pulse_events : &[PulseEvent]) -> (Real,Real) {
        baselined
            .evaluate_events(pulse_events)
            .fold((0.,0.),|(full_v,full_d), (_,v,d)|(full_v + v.abs(), full_d + d.abs()))
    }

    pub fn run_and_print_evaluation<'a>(&self, name : &str, baselined: impl Iterator<Item = (Real,Real)> + Clone, pulse_events : &[PulseEvent]) {
        println!("[{name} Evaluation]");
        let (v,d) = self.run_evaluation(name, baselined, pulse_events);
        println!("Area under trace curve:        {0:.2}", v);
        println!("Area under trace - simulation: {0:.2}", d);
        println!("[Evaluation Finished]");
    }

    pub fn save_baselined(&self, save_file_name: String, trace_baselined: impl Iterator<Item = (Real,Real)>) {
        trace_baselined
            .save_to_file(&(save_file_name + "_baselined" + ".csv")).unwrap();
    }

    pub fn save_smoothed(&self, save_file_name: String, trace_smoothed : impl Iterator<Item = (Real,Stats,OptFeedParam<Real>)>) {
        trace_smoothed
            .end_feedback()
            .map(tracedata::extract::enumerated_mean)
            .save_to_file(&(save_file_name + "_smoothed" + ".csv")).unwrap();
        
    }

    pub fn save_pulse_simulation(&self, save_file_name: String, trace_baselined: impl Iterator<Item = (Real,Real)> + Clone, pulse_events : &[PulseEvent]) {
        trace_baselined
            .clone()
            .to_trace(pulse_events)
            .save_to_file(&(save_file_name.clone() + "_simulated" + ".csv")).unwrap();
    }
        

    pub fn save_pulse_events(&self, save_file_name: String, pulse_events : Vec<PulseEvent>) {
        pulse_events.into_iter().save_to_file(&(save_file_name + "_pulses" + ".csv")).unwrap();
    }

    pub fn run_benchmark<'a>(&self, trace_smoothed: impl Iterator<Item = (Real,Stats,OptFeedParam<Real>)> + Clone) {
        
        println!("[Running Benchmarks]");
        //  Timing of trace_smoothed
        {
            let (_, time) = time_collect_vec(&trace_smoothed);
            println!(" - Trace preprocessed in {0} ms", time);
        };

        let pulse_events = trace_smoothed
            .clone()
            //.events_with_feedback(PulseDetector::new(LocalExtremeDetector::new()));
            .events_with_feedback(PulseDetector::new(ChangeDetector::new(self.advanced_parameters.change_detector_threshold), self.advanced_parameters.change_detector_bound));

        //  Timing of pulse_events
        let pulse_events_realised = {
            let (vec, time) = time_collect_vec(&pulse_events);
            println!(" - {0} pulses detected in {1:.3} ms, {2:.3} us/pulse", vec.len(), time, 1000.*time/vec.len() as Real);
            vec
        };

        let trace_simulated = trace_smoothed
            .clone()
            .end_feedback()
            .map(tracedata::extract::enumerated_mean)
            .to_trace(pulse_events_realised.as_slice());
        //  Timing of trace_simulated
        {
            let (_, time) = time_collect_vec(&trace_simulated);
            println!(" - Simulation created in {0} ms", time);
        }
        println!("[Benchmarks Finished]");
    }
}
