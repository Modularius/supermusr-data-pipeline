use crate::{
    parameters::{AdvancedMuonDetectorParameters, ConstantPhaseDiscriminatorParameters, Mode},
    pulse_detection::{
        advanced_muon_detector::{AdvancedMuonDetector, BasicMuonAssembler},
        threshold_detector::{ThresholdAssembler, ThresholdDetector, UpperThreshold},
        window::{Baseline, FiniteDifferences, SmoothingWindow, WindowFilter},
        AssembleFilter, EventFilter, Real, SaveToFileFilter,
    },
};
use std::path::{Path, PathBuf};
use supermusr_common::{Channel, EventData, FrameNumber, Intensity, Time};
use supermusr_common::{Channel, EventData, FrameNumber, Intensity, Time};
use supermusr_streaming_types::{
    dat1_digitizer_analog_trace_v1_generated::{ChannelTrace, DigitizerAnalogTraceMessage},
    dev1_digitizer_event_v1_generated::{
        finish_digitizer_event_list_message_buffer, DigitizerEventListMessage,
        DigitizerEventListMessageArgs,
    },
    flatbuffers::FlatBufferBuilder,
    frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args},
};
use tracing;

fn find_channel_events(
    metadata: &FrameMetadataV1,
    metadata: &FrameMetadataV1,
    trace: &ChannelTrace,
    sample_time: Real,
    mode: &Mode,
    save_options: Option<&Path>,
) -> (Vec<Time>, Vec<Intensity>) {
    match &mode {
        Mode::ConstantPhaseDiscriminator(parameters) => {
            find_constant_events(metadata, trace, sample_time, parameters, save_options)
        }
        Mode::AdvancedMuonDetector(parameters) => {
            find_advanced_events(metadata, trace, sample_time, parameters, save_options)
        }
    }
}

fn find_constant_events(
    metadata: &FrameMetadataV1,
    metadata: &FrameMetadataV1,
    trace: &ChannelTrace,
    sample_time : Real,
    parameters: &ConstantPhaseDiscriminatorParameters,
    save_path: Option<&Path>,
) -> (Vec<Time>, Vec<Intensity>) {
    let raw = trace
        .voltage()
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i as Real, -(v as Real)));

    let pulses = raw
        .clone()
        .events(ThresholdDetector::<UpperThreshold>::new(
            &parameters.threshold_trigger.0,
        ));

    if let Some(save_path) = save_path {
        raw.clone()
            .save_to_file(&get_save_file_name(save_path, metadata.frame_number(),trace.channel(), "raw"))
            .save_to_file(&get_save_file_name(save_path, metadata.frame_number(),trace.channel(), "raw"))
            .unwrap();

        pulses
            .clone()
            .save_to_file(&get_save_file_name(save_path, metadata.frame_number(), trace.channel(), "pulses"))
            .save_to_file(&get_save_file_name(save_path, metadata.frame_number(), trace.channel(), "pulses"))
            .unwrap();
    }

    let mut time = Vec::<Time>::new();
    let mut voltage = Vec::<Intensity>::new();
    for pulse in pulses {
        time.push(pulse.0 as Time);
        voltage.push(0 /*pulse.1. as Intensity*/);
    }
    (time,voltage)
}

fn find_advanced_events(
    metadata: &FrameMetadataV1,
    metadata: &FrameMetadataV1,
    trace: &ChannelTrace,
    sample_time : Real,
    parameters: &AdvancedMuonDetectorParameters,
    save_path: Option<&Path>,
) -> (Vec<Time>, Vec<Intensity>) {
    let raw = trace
        .voltage()
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i as Real, -(v as Real)));

    let smoothed = raw
        .clone()
        .window(Baseline::new(parameters.baseline_length.unwrap_or(0), 0.1))
        .window(SmoothingWindow::new(
            parameters.smoothing_window_size.unwrap_or(1),
        ))
        .map(|(i, stats)| (i, stats.mean));

    let events = smoothed
        .clone()
        .window(FiniteDifferences::<2>::new())
        .events(AdvancedMuonDetector::new(
            parameters.muon_onset,
            parameters.muon_fall,
            parameters.muon_termination,
            parameters.duration,
        ));

    let pulses = events
        .clone()
        .assemble(BasicMuonAssembler::default())
        .filter(|pulse| {
            Option::zip(parameters.min_amplitude, pulse.peak.value)
                .map(|(min, val)| min <= val)
                .unwrap_or(true)
        })
        .filter(|pulse| {
            Option::zip(parameters.max_amplitude, pulse.peak.value)
                .map(|(max, val)| max >= val)
                .unwrap_or(true)
        });

    if let Some(save_path) = save_path {
        raw.clone()
            .save_to_file(&get_save_file_name(save_path, metadata.frame_number(), trace.channel(), "raw"))
            .unwrap();

        smoothed
            .clone()
            .save_to_file(&get_save_file_name(save_path, metadata.frame_number(), trace.channel(), "smoothed"))
            .save_to_file(&get_save_file_name(save_path, metadata.frame_number(), trace.channel(), "smoothed"))
            .unwrap();

        pulses
            .clone()
            .save_to_file(&get_save_file_name(save_path, metadata.frame_number(), trace.channel(), "pulses"))
            .save_to_file(&get_save_file_name(save_path, metadata.frame_number(), trace.channel(), "pulses"))
            .unwrap();
    }

    let mut time = Vec::<Time>::new();
    let mut voltage = Vec::<Intensity>::new();
    for pulse in pulses {
        time.push(pulse.steepest_rise.time.unwrap_or_default() as Time);
        voltage.push(pulse.peak.value.unwrap_or_default() as Intensity);
    }
    (time,voltage)
}

fn get_save_file_name(path: &Path, frame_number: FrameNumber, channel: Channel, subscript: &str) -> PathBuf {
fn get_save_file_name(path: &Path, frame_number: FrameNumber, channel: Channel, subscript: &str) -> PathBuf {
    let file_name = format!(
        "{0}f{frame_number}c{channel}_{subscript}",
        "{0}f{frame_number}c{channel}_{subscript}",
        path.file_stem()
            .and_then(|os_str| os_str.to_str())
            .expect("file-name should be a valid file name")
    );
    match path.parent() {
        Some(parent) => parent.to_owned().join(file_name).with_extension("csv"),
        None => PathBuf::from(file_name).with_extension("csv"),
    }
}

#[tracing::instrument]
pub(crate) fn process<'a>(
    fbb : &mut FlatBufferBuilder<'a>,
    trace: &'a DigitizerAnalogTraceMessage,
    mode: &Mode,
    save_options: Option<&Path>,
) {
    log::info!(
        "Dig ID: {}, Metadata: {:?}",
        trace.digitizer_id(),
        trace.metadata()
    );

    let sample_time_in_ns: Real = 1_000_000_000.0 / trace.sample_rate() as Real;

    let events = std::thread::scope(|scope| {
        let mut events = EventData::default();
        let vec : Vec<(Channel,_)> = trace
            .channels()
            .unwrap()
            .iter()
            .map(|channel_trace|
                (channel_trace.channel(),
                    scope.spawn(move ||
                        find_channel_events(&trace.metadata(), &channel_trace, sample_time_in_ns, mode, save_options)
                    )
                )
            )
            .collect();
        for (channel,handle) in vec {
            let (time,voltage) = handle.join().unwrap();
            events.channel.extend_from_slice(&vec![channel; time.len()]);
            events.time.extend_from_slice(&time);
            events.voltage.extend_from_slice(&voltage);
        }
        events
    });
    let metadata = FrameMetadataV1Args {
        frame_number: trace.metadata().frame_number(),
        period_number: trace.metadata().period_number(),
        running: trace.metadata().running(),
        protons_per_pulse: trace.metadata().protons_per_pulse(),
        timestamp: trace.metadata().timestamp(),
        veto_flags: trace.metadata().veto_flags(),
    };
    let metadata = FrameMetadataV1::create(fbb, &metadata);

    let time = Some(fbb.create_vector(&events.time));
    let voltage = Some(fbb.create_vector(&events.voltage));
    let channel = Some(fbb.create_vector(&events.channel));

    let message = DigitizerEventListMessageArgs {
        digitizer_id: trace.digitizer_id(),
        metadata: Some(metadata),
        time,
        voltage,
        channel,
    };
    let message = DigitizerEventListMessage::create(fbb, &message);
    finish_digitizer_event_list_message_buffer(fbb, message);
}

#[cfg(test)]
mod tests {
    use crate::parameters::ThresholdDurationWrapper;
    use chrono::Utc;
    use std::str::FromStr;
    use supermusr_streaming_types::{
        dat1_digitizer_analog_trace_v1_generated::{
            finish_digitizer_analog_trace_message_buffer, root_as_digitizer_analog_trace_message,
            ChannelTraceArgs, DigitizerAnalogTraceMessage, DigitizerAnalogTraceMessageArgs,
        },
        dev1_digitizer_event_v1_generated::{
            digitizer_event_list_message_buffer_has_identifier,
            root_as_digitizer_event_list_message,
        },
        frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args, GpsTime},
    };

    use super::*;

    #[test]
    fn test_full_message() {
        let mut fbb = FlatBufferBuilder::new();

        let time: GpsTime = Utc::now().into();

        let metadata = FrameMetadataV1Args {
            frame_number: 0,
            period_number: 0,
            protons_per_pulse: 0,
            running: true,
            timestamp: Some(&time),
            veto_flags: 0,
        };
        let metadata = FrameMetadataV1::create(&mut fbb, &metadata);

        let channel0_voltage: Vec<u16> = vec![0, 1, 2, 1, 0, 1, 2, 1, 8, 0, 2, 8, 3, 1, 2];
        let channel0_voltage = Some(fbb.create_vector::<u16>(&channel0_voltage));
        let channel0 = ChannelTrace::create(
            &mut fbb,
            &ChannelTraceArgs {
                channel: 0,
                voltage: channel0_voltage,
            },
        );

        let message = DigitizerAnalogTraceMessageArgs {
            digitizer_id: 0,
            metadata: Some(metadata),
            sample_rate: 1_000_000_000, // 1 GS/s
            channels: Some(fbb.create_vector(&[channel0])),
        };
        let message = DigitizerAnalogTraceMessage::create(&mut fbb, &message);
        finish_digitizer_analog_trace_message_buffer(&mut fbb, message);

        let message = fbb.finished_data().to_vec();
        let message = root_as_digitizer_analog_trace_message(&message).unwrap();

        let test_parameters = ConstantPhaseDiscriminatorParameters {
            threshold_trigger: ThresholdDurationWrapper::from_str("5,1,0").unwrap(),
        };
        let mut fbb = FlatBufferBuilder::new();
        process(
            &mut fbb,
            &message,
            &Mode::ConstantPhaseDiscriminator(test_parameters),
            None,
        );

        assert!(digitizer_event_list_message_buffer_has_identifier(fbb.finished_data()));
        let event_message = root_as_digitizer_event_list_message(fbb.finished_data()).unwrap();

        assert_eq!(
            vec![0, 0],
            event_message.channel().unwrap().iter().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![8, 11],
            event_message.time().unwrap().iter().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![0, 0],
            event_message.voltage().unwrap().iter().collect::<Vec<_>>()
        );
    }
}
