mod borrowed_messages;
mod search_results;

use crate::sessions::SessionEngine;
use clap::Args;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) use borrowed_messages::{
    BorrowedMessageError, EventListMessage, FBMessage, TraceMessage,
};
pub(crate) use search_results::{Cache, SearchResults};

/// Encapsulates all run-time settings which are only available to the server.
#[derive(Default, Clone)]
pub struct ServerSideData {
    pub session_engine: Arc<Mutex<SessionEngine>>,
}

/// Contains the settings defined in the CLI used as default values in the UI's inputs.
#[derive(Default, Clone, Debug, Args)]
pub struct ServerIntervals {
    /// The frequency with which the server purges expired sessions.
    #[clap(long, default_value = "600")]
    pub purge_session_interval_sec: u64,

    /// Specifies the time-to-live of a user session. Any session whose time since last refresh is older than this is removed during a session purge cycle.
    #[clap(long, default_value = "600")]
    pub session_ttl_sec: i64,
}
