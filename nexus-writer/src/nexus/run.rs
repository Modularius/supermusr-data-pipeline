use crate::{
    error::{HDF5Error, NexusPushError, RunError}, schematic::{
        elements::{group::NexusGroup, traits::NexusPushMessage},
        groups::NXRoot,
    }
};

use super::{NexusSettings, RunParameters};
use chrono::{DateTime, Duration, Utc};
use hdf5::{File, FileBuilder};
use std::path::Path;
use supermusr_common::spanned::{SpanOnce, SpanOnceError, Spanned, SpannedAggregator, SpannedMut};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm,
    ecs_f144_logdata_generated::f144_LogData, ecs_pl72_run_start_generated::RunStart,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};
use tracing::{info_span, Span};

pub(crate) struct Run {
    span: SpanOnce,
    parameters: RunParameters,
    file: File,
    nx_root: NexusGroup<NXRoot>,
}

impl Run {
    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn new_run(
        filename: Option<&Path>,
        run_start: RunStart<'_>,
        nexus_settings: &NexusSettings,
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

        let run_started = nx_root.push_message(&run_start, &file).unwrap();
        Ok(Self {
            span: Default::default(),
            parameters: RunParameters::new(run_started),
            file,
            nx_root,
        })
    }

    //#[cfg(test)]  Uncomment this at a later stage
    pub(crate) fn parameters(&self) -> &RunParameters {
        &self.parameters
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_logdata_to_run(&mut self, logdata: &f144_LogData) -> anyhow::Result<()> {
        self.nx_root.push_message(logdata, &self.file)?;

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_alarm_to_run(&mut self, alarm: Alarm) -> anyhow::Result<()> {
        self.nx_root.push_message(&alarm, &self.file)?;
        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_selogdata(
        &mut self,
        selogdata: se00_SampleEnvironmentData,
    ) -> anyhow::Result<()> {
        self.nx_root.push_message(&selogdata, &self.file)?;

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_message(
        &mut self,
        message: &FrameAssembledEventListMessage,
    ) -> anyhow::Result<()> {
        self.nx_root
            .push_message(message, &self.file)?;
        self.parameters.new_frame();
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
        self.span()
            .get()?
            .in_scope(aggregated_span_fn)
            .follows_from(tracing::Span::current());
        Ok(())
    }

    fn end_span(&self) -> Result<(), SpanOnceError> {
        //let span_once = ;//.take().expect("SpanOnce should be takeable");
        self.span()
            .get()?
            .record("run_has_run_stop", self.has_run_stop());
        Ok(())
    }
}
