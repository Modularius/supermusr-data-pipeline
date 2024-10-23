use crate::{
    elements::{
        group::NexusGroup,
        traits::{NexusHandleMessage, NexusPushMessage, StandardMessage},
    },
    error::{
        HDF5Error, NexusConversionError, NexusMissingError, NexusMissingEventlistError,
        NexusPushError, RunError,
    },
    schematic::groups::NXRoot,
};

use super::{NexusConfiguration, NexusSettings, RunParameters};
use chrono::{DateTime, Duration, Utc};
use hdf5::{File, FileBuilder, Group};
use std::{fs::create_dir_all, future::Future, io, path::Path};
use supermusr_common::spanned::{SpanOnce, SpanOnceError, Spanned, SpannedAggregator, SpannedMut};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm,
    ecs_f144_logdata_generated::f144_LogData, ecs_pl72_run_start_generated::RunStart,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};
use tracing::{info, info_span, warn, Span};

pub(crate) struct FrameParameters {
    pub(crate) datetime: DateTime<Utc>,
}

pub(crate) struct PeriodParameters {}

pub(crate) struct Run {
    span: SpanOnce,
    parameters: RunParameters,
    frames: Vec<FrameParameters>,
    periods: Vec<PeriodParameters>,
    file: File,
    nx_root: NexusGroup<NXRoot>,
}
impl Run {
    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn new_run(
        filename: Option<&Path>,
        run_start: RunStart<'_>,
        nexus_settings: &NexusSettings,
        nexus_configuration: &NexusConfiguration,
    ) -> Result<Self, RunError> {
        let filename = {
            let mut filename = filename.expect("").to_owned();
            filename.push(run_start.run_name().unwrap());
            filename.set_extension("nxs");
            filename
        };

        let file = FileBuilder::new()
            .with_fapl(|fapl| {
                fapl.libver_bounds(
                    hdf5::file::LibraryVersion::V110,
                    hdf5::file::LibraryVersion::V110,
                )
            })
            .create(&filename)
            .map_err(HDF5Error::HDF5)?;
        {
            if nexus_settings.use_swmr {
                let err = unsafe { hdf5_sys::h5f::H5Fstart_swmr_write(file.id()) };
                if err != 0 {
                    Err(RunError::StartSWMRWriterError(err))?;
                }
            }
        }
        let mut nx_root = NexusGroup::new(
            filename
                .file_name()
                .ok_or(RunError::FileNameError)?
                .to_str()
                .ok_or(RunError::FileNameError)?,
            nexus_settings,
        );

        let run_started = nx_root
            .push_message(&run_start, &file)
            .expect("RunStart should be handled by nx_root");
        nx_root
            .push_message(nexus_configuration, &file)
            .expect("NexusConfiguration should be handled by nx_root");
        Ok(Self {
            span: Default::default(),
            parameters: RunParameters::new(run_started),
            file,
            nx_root,
            frames: Default::default(),
            periods: Default::default(),
        })
    }

    //#[cfg(test)]  Uncomment this at a later stage
    pub(crate) fn parameters(&self) -> &RunParameters {
        &self.parameters
    }

    #[tracing::instrument(skip_all, level = "info")]
    pub(crate) fn move_to_archive(
        &self,
        file_name: &Path,
        archive_name: &Path,
    ) -> io::Result<impl Future<Output = ()>> {
        create_dir_all(archive_name)?;
        let from_path = {
            let mut filename = file_name.to_owned();
            filename.push(&self.parameters.started.run_name);
            filename.set_extension("nxs");
            filename
        };
        let to_path = {
            let mut filename = archive_name.to_owned();
            filename.push(&self.parameters.started.run_name);
            filename.set_extension("nxs");
            filename
        };
        let span = tracing::Span::current();
        let future = async move {
            info_span!(parent: &span, "move-async").in_scope(|| {
                match std::fs::copy(from_path.as_path(), to_path) {
                    Ok(bytes) => info!("File Move Succesful. {bytes} byte(s) moved."),
                    Err(e) => warn!("File Move Error {e}"),
                }
                if let Err(e) = std::fs::remove_file(from_path) {
                    warn!("Error removing temporary file: {e}");
                }
            });
        };
        Ok(future)
    }

    #[tracing::instrument(skip_all, level = "info")]
    pub(crate) fn finish(&mut self) -> Result<(), NexusPushError> {
        self.nx_root.push_message(&self.periods, &self.file)?;
        // Ensure we don't attempt to take a minimum from an empty list
        if !self.frames.is_empty() {
            self.nx_root.push_message(&self.frames, &self.file)?;
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn get_name(&self) -> &str {
        &self.parameters.started.run_name
    }

    pub(crate) fn has_run_stop(&self) -> bool {
        self.parameters.collect_until.is_some()
    }

    pub(crate) fn set_stop_if_valid(&mut self, run_stop: RunStop<'_>) -> anyhow::Result<()> {
        let bounded = self.nx_root.push_message(&run_stop, &self.file)?;
        self.parameters.bound(bounded)?;
        Ok(())
    }

    pub(crate) fn is_message_timestamp_valid(&self, timestamp: &DateTime<Utc>) -> bool {
        self.parameters.is_message_timestamp_valid(timestamp)
    }

    pub(crate) fn has_completed(&self, delay: &Duration) -> bool {
        self.parameters
            .collect_until
            .is_some_and(|_| Utc::now() - self.parameters.last_modified > *delay)
    }
}

impl StandardMessage<NexusConfiguration> for Run {}
impl<'a> StandardMessage<RunStop<'a>> for Run {}
impl<'a> StandardMessage<Alarm<'a>> for Run {}
impl<'a> StandardMessage<se00_SampleEnvironmentData<'a>> for Run {}
impl<'a> StandardMessage<f144_LogData<'a>> for Run {}

impl<'a> NexusPushMessage<FrameAssembledEventListMessage<'a>, ()> for Run {
    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    fn push_message(
        &mut self,
        message: &FrameAssembledEventListMessage<'a>,
        _: &(),
    ) -> Result<(), NexusPushError> {
        let datetime: DateTime<Utc> = message
            .metadata()
            .timestamp()
            .copied()
            .ok_or(NexusMissingError::Eventlist(
                NexusMissingEventlistError::Timestamp,
            ))?
            .try_into()
            .map_err(NexusConversionError::GpsTimeConversion)?;

        self.frames.push(FrameParameters { datetime });
        self.nx_root.push_message(message, &self.file)?;
        self.parameters.update_last_modified();
        Ok(())
    }
}

impl<M, R> NexusPushMessage<M, (), R> for Run
where
    Self: StandardMessage<M>,
    NXRoot: NexusHandleMessage<M, Group, R>,
{
    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    fn push_message(&mut self, message: &M, _: &()) -> Result<R, NexusPushError> {
        let result = self.nx_root.push_message(message, &self.file)?;
        self.parameters.update_last_modified();
        Ok(result)
    }
}

impl Spanned for Run {
    fn span(&self) -> &SpanOnce {
        &self.span
    }
}

impl SpannedMut for Run {
    fn span_mut(&mut self) -> &mut SpanOnce {
        &mut self.span
    }
}

impl SpannedAggregator for Run {
    fn span_init(&mut self) -> Result<(), SpanOnceError> {
        let span = info_span!(target: "otel", parent: None,
            "Run",
            "run_name" = self.parameters.started.run_name.as_str(),
            "run_has_run_stop" = tracing::field::Empty
        );
        self.span_mut().init(span)
    }

    fn link_current_span<F: Fn() -> Span>(
        &self,
        aggregated_span_fn: F,
    ) -> Result<(), SpanOnceError> {
        self.span().get().map(|span| {
            span.in_scope(aggregated_span_fn)
                .follows_from(tracing::Span::current());
        })
    }

    fn end_span(&self) -> Result<(), SpanOnceError> {
        self.span()
            .get()?
            .record("run_has_run_stop", self.has_run_stop());
        Ok(())
    }
}
