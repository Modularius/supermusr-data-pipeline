pub(crate) mod alarm;
pub(crate) mod create_messages;
pub(crate) mod runlog;
pub(crate) mod sample_environment;

use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};

#[derive(Clone, Parser)]
pub(crate) struct Start {
    /// Topic to publish command to
    #[clap(long)]
    topic: String,

    /// Timestamp of the command, defaults to now, if not given.
    #[clap(long)]
    time: Option<DateTime<Utc>>,

    /// Unique name of the run
    #[clap(long)]
    run_name: String,

    /// Name of the instrument being run
    #[clap(long)]
    instrument_name: String,
}
#[derive(Clone, Parser)]
pub(crate) struct Stop {
    /// Topic to publish command to
    #[clap(long)]
    topic: String,

    /// Timestamp of the command, defaults to now, if not given.
    #[clap(long)]
    time: Option<DateTime<Utc>>,

    /// Unique name of the run
    #[clap(long)]
    run_name: String,
}

#[derive(Clone, Debug, Parser)]
pub(crate) struct RunLogData {
    /// Topic to publish command to
    #[clap(long)]
    topic: String,

    /// Timestamp of the command, defaults to now, if not given.
    #[clap(long)]
    time: Option<DateTime<Utc>>,

    /// Name of the source being logged
    #[clap(long)]
    source_name: String,

    /// Type of the logdata
    #[clap(long)]
    value_type: String,

    /// Value of the logdata
    #[clap()]
    value: Vec<String>,
}

#[derive(Clone, Debug, Parser)]
pub(crate) struct SampleEnvData {
    /// Topic to publish command to
    #[clap(long)]
    topic: String,

    /// Timestamp of the command, defaults to now, if not given.
    #[clap(long)]
    time: Option<DateTime<Utc>>,

    /// Name of the source being logged
    #[clap(long)]
    name: String,

    #[clap(long)]
    channel: Option<i32>,

    /// Optional: time between each sample in ns
    #[clap(long)]
    time_delta: Option<f64>,

    /// Type of the sample value
    #[clap(long, default_value = "int64")]
    values_type: String,

    /// Incrementing counter
    #[clap(long)]
    message_counter: Option<i64>,

    #[clap(long)]
    location: String,

    /// Value of samples
    #[clap()]
    values: Vec<String>,

    #[command(subcommand)]
    timestamps: Option<SampleEnvTimestamp>,
}

#[derive(Clone, Debug, Subcommand)]
enum SampleEnvTimestamp {
    Timestamps(SampleEnvTimestampData),
}
#[derive(Clone, Debug, Parser)]
pub(crate) struct SampleEnvTimestampData {
    #[clap()]
    timestamps: Vec<DateTime<Utc>>,
}

#[derive(Clone, Debug, Parser)]
pub(crate) struct AlarmData {
    /// Topic to publish command to
    #[clap(long)]
    topic: String,

    /// Timestamp of the command, defaults to now, if not given.
    #[clap(long)]
    time: Option<DateTime<Utc>>,

    /// Source Name
    #[clap(long)]
    source_name: String,

    #[clap(long)]
    severity: String,

    #[clap(long)]
    message: String,
}