//! This module allows one to simulate instances of DigitizerAnalogTraceMessage
//! using the FlatBufferBuilder.
//!
use anyhow::{Result, Error};
use chrono::Utc;
//use std::ops::Range;
use flatbuffers::FlatBufferBuilder;
use std::time::Duration;
use rdkafka::{producer::{FutureProducer, FutureRecord}, util::Timeout};

use streaming_types::frame_metadata_v1_generated::GpsTime;

pub mod random;
pub use random::RandomTraceMessage;


pub mod messages;

pub trait MessageGenerator {
    fn create_message(
        fbb: &mut FlatBufferBuilder<'_>,
        time: GpsTime,
        frame_number: u32,
        digitizer_id: u8,
        measurements_per_frame: usize,
        num_channels: usize,
        data: &Self,
    ) -> Result<String, Error>;
}








pub async fn dispatch_message(fbb : &FlatBufferBuilder<'_>, producer : &FutureProducer, topic : &str, timeout_ms : u64, key : &str) -> Result<()>
{
    let future_record = FutureRecord::to(topic).payload(fbb.finished_data()).key(key);
    let timeout = Timeout::After(Duration::from_millis(timeout_ms));
    match producer.send(future_record,timeout).await
    {
        Ok(r) => log::debug!("Delivery: {:?}", r),
        Err(e) => log::error!("Delivery failed: {:?}", e.0),
    };
    Ok(())
}



