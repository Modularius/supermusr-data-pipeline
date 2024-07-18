mod daq_trace;
mod message_debug;
mod frame_debug;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Provides metrics regarding data transmission from the digitisers via Kafka.
    #[clap(name = "daq-trace")]
    DaqTrace(DaqTraceOpts),

    /// Run message dumping tool.
    #[clap(name = "message-debug")]
    MessageDebug(CommonOpts),

    /// Frame message dumping tool.
    #[clap(name = "frame-debug")]
    FrameDebug(FrameOpts),
}

#[derive(Debug, Args)]
struct CommonOpts {
    #[clap(long)]
    broker: String,

    #[clap(long)]
    username: Option<String>,

    #[clap(long)]
    password: Option<String>,

    #[clap(long = "group")]
    consumer_group: String,

    #[clap(long)]
    topic: String,
}

#[derive(Debug, Args)]
struct FrameOpts {
    #[clap(long)]
    extra_topic: String,

    #[clap(long)]
    frames_to_collect: usize,

    #[clap(flatten)]
    common: CommonOpts,
}

#[derive(Debug, Args)]
struct DaqTraceOpts {
    #[clap(long, default_value_t = 5)]
    message_rate_interval: u64,

    #[clap(flatten)]
    common: CommonOpts,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::DaqTrace(args) => daq_trace::run(args).await,
        Commands::MessageDebug(args) => message_debug::run(args).await,
        Commands::FrameDebug(args) => frame_debug::run(args).await,
    }
}
