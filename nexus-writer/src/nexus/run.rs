use crate::schematic::{elements::NexusPushMessageWithContext, Nexus};

use super::{NexusSettings, RunParameters};
use chrono::{DateTime, Duration, Utc};
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
    nexus: Nexus,
    num_frames: usize,
}

impl Run {
    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn new_run(
        filename: Option<&Path>,
        run_start: RunStart<'_>,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<Self> {
        /*if let Some(filename) = filename {
            let mut hdf5 = RunFile::new_runfile(filename, &parameters.run_name, nexus_settings)?;
            hdf5.init(&parameters)?;
            hdf5.close()?;
        }*/
        let filename = {
            let mut filename = filename.expect("").to_owned();
            filename.push(run_start.run_name().unwrap());
            filename.set_extension("nxs");
            filename
        };
        let mut nexus = Nexus::new(&filename, nexus_settings)?;
            //nexus.create()?;
        let parameters = nexus.push_message(&run_start)?;
        Ok(Self {
            span: Default::default(),
            parameters,
            nexus,
            num_frames: usize::default(),
        })
    }
    //#[cfg(test)]  Uncomment this at a later stage
    pub(crate) fn parameters(&self) -> &RunParameters {
        &self.parameters
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_logdata_to_run(
        &mut self,
        filename: Option<&Path>,
        logdata: &f144_LogData,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<()> {
        /*if let Some(filename) = filename {
            let mut hdf5 = RunFile::open_runfile(filename, &self.parameters.run_name)?;
            hdf5.push_logdata_to_runfile(logdata, nexus_settings)?;
            hdf5.close()?;
        }*/
        self.nexus.push_message(logdata)?;

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_alarm_to_run(
        &mut self,
        filename: Option<&Path>,
        alarm: Alarm,
    ) -> anyhow::Result<()> {
        /*if let Some(filename) = filename {
            let mut hdf5 = RunFile::open_runfile(filename, &self.parameters.run_name)?;
            hdf5.push_alarm_to_runfile(alarm)?;
            hdf5.close()?;
        }*/

        self.nexus.push_message(&alarm)?;

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_selogdata(
        &mut self,
        filename: Option<&Path>,
        selogdata: se00_SampleEnvironmentData,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<()> {
        /*if let Some(filename) = filename {
            let mut hdf5 = RunFile::open_runfile(filename, &self.parameters.run_name)?;
            hdf5.push_selogdata(selogdata, nexus_settings)?;
            hdf5.close()?;
        }*/

        self.nexus.push_message(&selogdata)?;

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_message(
        &mut self,
        filename: Option<&Path>,
        message: &FrameAssembledEventListMessage,
    ) -> anyhow::Result<()> {
        self.nexus.push_message_with_context(message, &mut self.parameters)?;

        self.num_frames += 1;
        self.parameters.update_last_modified();
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn get_name(&self) -> &str {
        &self.parameters.run_name
    }

    pub(crate) fn has_run_stop(&self) -> bool {
        self.parameters.collect_until.is_some()
    }

    pub(crate) fn set_stop_if_valid(
        &mut self,
        filename: Option<&Path>,
        run_stop: RunStop<'_>,
    ) -> anyhow::Result<()> {
        self.parameters.set_stop_if_valid(run_stop)?;

        self.nexus.push_message(&run_stop)?;

        /*if let Some(filename) = filename {
            let mut hdf5 = RunFile::open_runfile(filename, &self.parameters.run_name)?;

            hdf5.set_end_time(
                &self
                    .parameters
                    .run_stop_parameters
                    .as_ref()
                    .expect("RunStopParameters exists") // This never panics
                    .collect_until,
            )?;
            hdf5.close()?;
        }*/
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
            "run_name" = self.parameters.run_name.as_str(),
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
