use chrono::{DateTime, TimeDelta, Utc};
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::RunStop, ecs_pl72_run_start_generated::RunStart,
};

use crate::error::{NexusConversionError, NexusMissingError, NexusMissingRunStartError, NexusPushError, RunStartError, RunStopError};

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
    pub(crate) fn new<'a>(message: &RunStart<'a>) -> Result<Self,RunStartError> {
        let collect_from = DateTime::<Utc>::from_timestamp_millis(message.start_time()
                .try_into()
                .map_err(NexusConversionError::TryFromInt)?
            ).ok_or(RunStartError::CollectFrom)?;
        let run_name = message.run_name()
            .ok_or(NexusMissingRunStartError::RunName)
            .map_err(NexusMissingError::RunStart)?
            .to_owned();
        Ok(Self { collect_from, run_name })
    }
}

pub(crate) trait RunBounded : Sized {
    fn new<'a>(message: &RunStop<'a>) -> Result<Self,RunStopError>;
}

impl RunBounded for DateTime<Utc> {
    fn new<'a>(message: &RunStop<'a>) -> Result<Self,RunStopError> {
        DateTime::<Utc>::from_timestamp_millis(
            message.stop_time()
                .try_into()
                .map_err(NexusConversionError::TryFromInt)?
        ).ok_or(RunStopError::CollectUntil)
    }
}

#[derive(Debug)]
pub(crate) struct RunParameters {
    pub(crate) started : RunStarted,
    pub(crate) collect_until : Option<DateTime<Utc>>,
    pub(crate) last_modified: DateTime<Utc>,
    pub(crate) num_frames: usize,   // Do we actually need this?
}

impl RunParameters {
    pub(super) fn new(started: RunStarted) -> Self {
        Self { started, collect_until: None, last_modified: Utc::now(), num_frames: 0 }
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn bound(&mut self, collect_until : DateTime<Utc>) -> Result<(), RunStopError> {
        if self.collect_until.is_some() {
            Err(RunStopError::UnexpectedRunStop)
        } else {
            if self.started.collect_from < collect_until {
                self.collect_until = Some(collect_until);
                self.update_last_modified();
                Ok(())
            } else {
                Err(RunStopError::RunStopBeforeRunStart)
            }
        }
    }

    #[tracing::instrument(skip_all, level = "trace")]
    pub(crate) fn new_frame(&mut self) {
        self.num_frames += 1;
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

/*
#[derive(Debug)]
pub(crate) struct RunParameters {
    pub(crate) collect_from: DateTime<Utc>,
    pub(crate) collect_until: Option<DateTime<Utc>>,
    pub(crate) last_modified: DateTime<Utc>,
    pub(crate) num_frames: usize,
    //pub(crate) run_stop_parameters: Option<RunStopParameters>,
    //pub(crate) num_periods: u32,
    pub(crate) run_name: String,
    //pub(crate) run_number: u32,
    //pub(crate) instrument_name: String,
}
impl RunParameters {
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn new(data: &RunStart<'_>) -> Result<Self, RunStartError> {
        let collect_from = DateTime::<Utc>::from_timestamp_millis(data.start_time()
            .try_into()
            .map_err(NexusConversionError::TryFromInt)?
        ).ok_or(RunStartError::CollectFrom)?;

        Ok(Self {
            collect_from,
            collect_until: None,
            num_frames: Default::default(),
            last_modified: Utc::now(),
            run_name: data.run_name()
                .ok_or(NexusMissingRunStartError::RunName)
                .map_err(NexusMissingError::RunStart)?
                .to_owned(),
        })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn set_stop_if_valid(&mut self, data: RunStop<'_>) -> Result<(), RunStopError> {
        if self.collect_until.is_some() {
            Err(RunStopError::UnexpectedRunStop)
        } else {
            let now = DateTime::<Utc>::from_timestamp_millis(
                data.stop_time()
                    .try_into()
                    .map_err(NexusConversionError::TryFromInt)?,
            )
            .ok_or(RunStopError::CollectUntil)?;

            if self.collect_from < now {
                self.collect_until = Some(now);
                self.update_last_modified();
                Ok(())
            } else {
                Err(RunStopError::RunStopBeforeRunStart)
            }
        }
    }

    /// Returns true if timestamp is strictly after collect_from and,
    /// if run_stop_parameters exist then, if timestamp is strictly
    /// before params.collect_until.
    #[tracing::instrument(skip_all, level = "trace")]
    pub(crate) fn is_message_timestamp_valid(&self, timestamp: &DateTime<Utc>) -> bool {
        if self.collect_from < *timestamp {
            self.collect_until
                .as_ref()
                .map(|collect_until| timestamp < collect_until)
                .unwrap_or(true)
        } else {
            false
        }
    }

    #[tracing::instrument(skip_all, level = "trace")]
    pub(crate) fn update_last_modified(&mut self) {
        self.last_modified = Utc::now();
    }
}

*/
/*
impl RunParameters {
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn new(data: RunStart<'_>, run_number: u32) -> anyhow::Result<Self> {
        Ok(Self {
            collect_from: DateTime::<Utc>::from_timestamp_millis(data.start_time().try_into()?)
                .ok_or(anyhow::anyhow!(
                    "Cannot create start_time from {0}",
                    &data.start_time()
                ))?,
            run_stop_parameters: None,
            num_periods: data.n_periods(),
            run_name: data
                .run_name()
                .ok_or(anyhow::anyhow!("Run Name not found"))?
                .to_owned(),
            run_number,
            instrument_name: data
                .instrument_name()
                .ok_or(anyhow::anyhow!("Instrument Name not found"))?
                .to_owned(),
        })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn set_stop_if_valid(&mut self, data: RunStop<'_>) -> anyhow::Result<()> {
        if self.run_stop_parameters.is_some() {
            Err(anyhow::anyhow!("Stop Command before Start Command"))
        } else {
            let stop_time =
                DateTime::<Utc>::from_timestamp_millis(data.stop_time().try_into()?).ok_or(
                    anyhow::anyhow!("Cannot create end_time from {0}", data.stop_time()),
                )?;
            if self.collect_from < stop_time {
                self.run_stop_parameters = Some(RunStopParameters {
                    collect_until: stop_time,
                    last_modified: Utc::now(),
                });
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "Stop Time earlier than current Start Time."
                ))
            }
        }
    }

    /// Returns true if timestamp is strictly after collect_from and,
    /// if run_stop_parameters exist then, if timestamp is strictly
    /// before params.collect_until.
    #[tracing::instrument(skip_all, level = "trace")]
    pub(crate) fn is_message_timestamp_valid(&self, timestamp: &DateTime<Utc>) -> bool {
        if self.collect_from < *timestamp {
            self.run_stop_parameters
                .as_ref()
                .map(|params| *timestamp < params.collect_until)
                .unwrap_or(true)
        } else {
            false
        }
    }

    #[tracing::instrument(skip_all, level = "trace")]
    pub(crate) fn update_last_modified(&mut self) {
        if let Some(params) = &mut self.run_stop_parameters {
            params.last_modified = Utc::now();
        }
    }
}
 */
