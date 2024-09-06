use crate::schematic::{elements::group::{NxPushMessage, NxPushMessageMut}, Nexus};

use super::{NexusSettings, RunParameters};
use chrono::{DateTime, Duration, Utc};
use std::path::Path;
use supermusr_common::spanned::{SpanOnce, SpanOnceError, Spanned, SpannedAggregator, SpannedMut};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage, ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm, ecs_f144_logdata_generated::f144_LogData, ecs_pl72_run_start_generated::RunStart, ecs_se00_data_generated::se00_SampleEnvironmentData
};
use tracing::{info_span, Span};

pub(crate) struct Run {
    span: SpanOnce,
    parameters: RunParameters,
    nexus: Option<Nexus>,
}

impl Run {
    #[tracing::instrument]
    pub(crate) fn new_run(
        filename: Option<&Path>,
        run_start: RunStart<'_>,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<Self> {
        let parameters = RunParameters::new(run_start, 0)?;
        /*if let Some(filename) = filename {
            let mut hdf5 = RunFile::new_runfile(filename, &parameters.run_name, nexus_settings)?;
            hdf5.init(&parameters)?;
            hdf5.close()?;
        }*/
        let nexus = {
            if let Some(filename) = filename {
                let filename = {
                    let mut filename = filename.to_owned();
                    filename.push(&parameters.run_name);
                    filename.set_extension("nxs");
                    filename
                };
                let mut nexus = Nexus::new(&filename, nexus_settings)?;
                nexus.create()?;
                nexus.get_root().push_message(&run_start)?;
                nexus.close()?;
                Some(nexus)
            } else {
                None
            }
        };
        Ok(Self {
            span: Default::default(),
            parameters,
            nexus
        })
    }
    //#[cfg(test)]  Uncomment this at a later stage
    pub(crate) fn parameters(&self) -> &RunParameters {
        &self.parameters
    }

    #[tracing::instrument(skip(self))]
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

        if let Some(ref mut nexus) = self.nexus {
            nexus.open()?;
            nexus.get_root_mut().push_message_mut(logdata)?;
            nexus.close()?;
        };

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(skip(self))]
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

        if let Some(ref mut nexus) = self.nexus {
            nexus.open()?;
            nexus.get_root_mut().push_message_mut(&alarm)?;
            nexus.close()?;
        }

        self.parameters.update_last_modified();
        Ok(())
    }

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

        if let Some(ref mut nexus) = self.nexus {
            nexus.open()?;
            nexus.get_root_mut().push_message_mut(&selogdata)?;
            nexus.close()?;
        }

        self.parameters.update_last_modified();
        Ok(())
    }

    pub(crate) fn push_message(
        &mut self,
        filename: Option<&Path>,
        message: &FrameAssembledEventListMessage,
    ) -> anyhow::Result<()> {
        /*if let Some(filename) = filename {
            let mut hdf5 = RunFile::open_runfile(filename, &self.parameters.run_name)?;
            hdf5.push_message_to_runfile(&self.parameters, message)?;
            hdf5.close()?;
        }*/

        if let Some(ref mut nexus) = self.nexus {
            nexus.open()?;
            nexus.get_root().push_message(message)?;
            nexus.close()?;
        }

        self.parameters.update_last_modified();
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn get_name(&self) -> &str {
        &self.parameters.run_name
    }

    pub(crate) fn has_run_stop(&self) -> bool {
        self.parameters.run_stop_parameters.is_some()
    }

    pub(crate) fn set_stop_if_valid(
        &mut self,
        filename: Option<&Path>,
        run_stop: RunStop<'_>,
    ) -> anyhow::Result<()> {
        self.parameters.set_stop_if_valid(run_stop)?;

        if let Some(ref mut nexus) = self.nexus {
            nexus.open()?;
            nexus.get_root().push_message(&run_stop)?;
            nexus.close()?;
        }

        /*if let Some(filename) = filename {
            let mut hdf5 = RunFile::open_runfile(filename, &self.parameters.run_name)?;

            hdf5.set_end_time(
                &self
                    .parameters
                    .run_stop_parameters
                    .as_ref()
                    .unwrap()
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
            .run_stop_parameters
            .as_ref()
            .map(|run_stop_parameters| Utc::now() - run_stop_parameters.last_modified > *delay)
            .unwrap_or(false)
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
        let span = info_span!(target: "otel", parent: None, "Run");
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
