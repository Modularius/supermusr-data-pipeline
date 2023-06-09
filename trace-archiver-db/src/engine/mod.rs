use std::ops::Div;

use anyhow::{anyhow, Result};
use streaming_types::dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage;
use async_trait::async_trait;

pub mod tdengine;
pub mod influxdb;
pub mod framedata;

#[async_trait]
pub trait TimeSeriesEngine {
    async fn process_message(&mut self, msg: &DigitizerAnalogTraceMessage) -> Result<()>;
    async fn post_message(&mut self) -> Result<String>;
}