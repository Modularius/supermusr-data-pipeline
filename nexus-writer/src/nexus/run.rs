use crate::{
    elements::{
        group::NexusGroup,
        traits::{NexusHandleMessage, NexusPushMessage},
    },
    error::{HDF5Error, NexusPushError, RunError},
    schematic::groups::NXRoot,
};

use super::{NexusSettings, RunParameters};
use chrono::{DateTime, Duration, Utc};
use hdf5::{File, FileBuilder};
use std::path::Path;
use supermusr_common::spanned::{SpanOnce, SpanOnceError, Spanned, SpannedAggregator, SpannedMut};
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::RunStop, ecs_pl72_run_start_generated::RunStart,
};
use tracing::{info_span, Span};

struct RunPeriods {
    period_type: usize,
    number: usize,
    raw_frames: usize,
    good_frames: usize,
}

pub(crate) struct Run {
    span: SpanOnce,
    parameters: RunParameters,
    period: Vec<RunPeriods>,
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
            period: Default::default()
        })
    }

    //#[cfg(test)]  Uncomment this at a later stage
    pub(crate) fn parameters(&self) -> &RunParameters {
        &self.parameters
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_message<M>(&mut self, message: &M) -> Result<(), NexusPushError>
    where
        NXRoot: NexusHandleMessage<M>,
    {
        self.nx_root.push_message(message, &self.file)?;
        self.parameters.update_last_modified();
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
