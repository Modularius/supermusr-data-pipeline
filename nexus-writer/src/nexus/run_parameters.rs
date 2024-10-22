use chrono::{DateTime, Utc};
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::RunStop, ecs_pl72_run_start_generated::RunStart,
};

use crate::error::{
    NexusConversionError, NexusMissingError, NexusMissingRunStartError, RunStartError, RunStopError,
};

/*#[derive(Default, Debug)]
pub(crate) struct RunStopParameters {
    pub(crate) collect_until: DateTime<Utc>,
    pub(crate) last_modified: DateTime<Utc>,
}*/

#[derive(Debug)]
pub(crate) struct RunStarted {
    pub(crate) collect_from: DateTime<Utc>,
    pub(crate) run_name: String,
}

impl RunStarted {
    pub(crate) fn new(message: &RunStart<'_>) -> Result<Self, RunStartError> {
        let collect_from = DateTime::<Utc>::from_timestamp_millis(
            message
                .start_time()
                .try_into()
                .map_err(NexusConversionError::TryFromInt)?,
        )
        .ok_or(RunStartError::CollectFrom)?;
        let run_name = message
            .run_name()
            .ok_or(NexusMissingRunStartError::RunName)
            .map_err(NexusMissingError::RunStart)?
            .to_owned();
        Ok(Self {
            collect_from,
            run_name,
        })
    }
}

pub(crate) trait RunBounded: Sized {
    fn new(message: &RunStop<'_>) -> Result<Self, RunStopError>;
}

impl RunBounded for DateTime<Utc> {
    fn new(message: &RunStop<'_>) -> Result<Self, RunStopError> {
        DateTime::<Utc>::from_timestamp_millis(
            message
                .stop_time()
                .try_into()
                .map_err(NexusConversionError::TryFromInt)?,
        )
        .ok_or(RunStopError::CollectUntil)
    }
}

#[derive(Debug)]
pub(crate) struct RunParameters {
    pub(crate) started: RunStarted,
    pub(crate) collect_until: Option<DateTime<Utc>>,
    pub(crate) last_modified: DateTime<Utc>,
}

impl RunParameters {
    pub(super) fn new(started: RunStarted) -> Self {
        Self {
            started,
            collect_until: None,
            last_modified: Utc::now(),
        }
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn bound(&mut self, collect_until: DateTime<Utc>) -> Result<(), RunStopError> {
        if self.collect_until.is_some() {
            Err(RunStopError::UnexpectedRunStop)
        } else if self.started.collect_from < collect_until {
            self.collect_until = Some(collect_until);
            self.update_last_modified();
            Ok(())
        } else {
            Err(RunStopError::RunStopBeforeRunStart)
        }
    }

    #[tracing::instrument(skip_all, level = "trace")]
    pub(crate) fn new_frame(&mut self) {
        self.update_last_modified();
    }

    #[tracing::instrument(skip_all, level = "trace")]
    pub(crate) fn update_last_modified(&mut self) {
        self.last_modified = Utc::now();
    }

    /// Returns true if timestamp is strictly after collect_from and,
    /// if run_stop_parameters exist then, if timestamp is strictly
    /// before params.collect_until.
    #[tracing::instrument(skip_all, level = "trace")]
    pub(crate) fn is_message_timestamp_valid(&self, timestamp: &DateTime<Utc>) -> bool {
        if self.started.collect_from < *timestamp {
            self.collect_until
                .as_ref()
                .map(|collect_until| timestamp < collect_until)
                .unwrap_or(true)
        } else {
            false
        }
    }
}
