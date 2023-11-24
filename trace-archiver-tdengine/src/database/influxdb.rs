use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc, Timelike};
use common::{Channel, DigitizerId, FrameNumber, Intensity};
use influxdb::{Client, InfluxDbWriteable, ReadQuery, WriteQuery};
use log::debug;
use streaming_types::dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage;

use super::{framedata::FrameData, TimeSeriesEngine};

//  Modify the FrameData struct to add influxdb functionality
impl FrameData {
    /// Create an influxdb Measurement instance with the given channel number and voltage
    /// #Arguments
    /// * `channel_number` - The index of the channel
    /// * `index` - The index of the measurement in the frame
    /// * `voltage` - The voltage of the measurement
    /// #Returns
    /// A Measurement instance
    fn make_measurement(
        &self,
        channel_number: Channel,
        index: usize,
        voltage: Intensity,
    ) -> Measurement {
        Measurement {
            time: self.calc_measurement_time(index),
            digitizer_id: self.digitizer_id,
            frame_number: self.frame_number,
            channel: channel_number,
            intensity: voltage as i32,
        }
    }
}

/// A structure representing an influxdb measurement, as it derives InfluxDbWriteable
/// it implements the WriteQuery function to send the measurement to the influxdb server.
/// #Fields
/// *time - a DateTime representing the measurement time.
/// *digitizer_id - the id of the digitizer marked as a tag.
/// *frame_number - the number of the frame marked as a tag.
/// *channel - the index of the channel marked as a tag.
/// *intesity - the intensity of the measurement, the sole field of the measurement.
#[derive(InfluxDbWriteable, Default)]
struct Measurement {
    time: DateTime<Utc>,
    #[influxdb(tag)]
    digitizer_id: DigitizerId,
    #[influxdb(tag)]
    frame_number: FrameNumber,
    #[influxdb(tag)]
    channel: Channel,
    #[doc = "Using type `Intensity` causes an error"]
    intensity: i32, //using type Intensity causes an error
}


/// A structure representing the influxdb engine.
/// #Fields
/// *client - a DateTime representing the measurement time.
/// *frame_data - the id of the digitizer marked as a tag.
/// *measurements - a vector of consisting of the measurements to write to the influxdb server.
pub(crate) struct InfluxDBEngine {
    url: String,
    database: String,
    client: Client,
    frame_data: FrameData,
    measurements: Vec<WriteQuery>,
    batch_size: usize,
    num_batches: usize,
    num_channels: usize,
}

impl InfluxDBEngine {
    /// Creates a new instance of InfluxDBEngine
    /// #Returns
    /// An instance connected to "http://localhost:8086" and database "TraceLogs".
    /// The token used to authenticate with the influxdb server is currently hardcoded.
    pub async fn new(
        broker: String,
        username: Option<String>,
        password: Option<String>,
        database: String,
        num_channels: usize,
        batch_size: usize,
    ) -> Self {
        let protocol = "http";
        
        let url = format!("{protocol}://{broker}");

        debug!("Creating InfluxDBEngine with url {url}");
        InfluxDBEngine {
            url: url.clone(),
            database: database.clone(),
            client: Client::new(url, database).with_token("e8ltJ5EW-eDdRDRa3B-nmv9MOtg2W9bC6tlvc0kMbMTTWDZW13Rm4JB9TFW9pWIQfwRMhm_17UH3N2mWmWFzUQ=="),//("UQaZ1HOeyP9hsT6DVwSWhxYcig3M4G0MYaFapTZ2j6Me6wuzNrLJmcw7lW8U-9KuKTJepjUBsrePRx3d5bCjdg=="),
            frame_data: FrameData::default(),
            measurements: Vec::<WriteQuery>::default(),
            batch_size,
            num_channels,
            num_batches: usize::default(),
        }
    }

    /// Clears all data from database "TraceLogs" and resets it.
    /// #Returns
    /// An emtpy result or an error arrising from the influxdb queries.
    pub async fn reset_database(&self) -> Result<()> {
        self.client
            .query(&ReadQuery::new(format!("DROP DATABASE {}", self.database)))
            .await?;
        self.client
            .query(&ReadQuery::new(format!("CREATE DATABASE {}", self.database)))
            .await?;
        Ok(())
    }
}

#[async_trait]
impl TimeSeriesEngine for InfluxDBEngine {
    /// Takes a reference to a ``DigitizerAnalogTraceMessage`` instance and extracts the relevant data from it.
    /// The user should then call ``post_message`` to send the data to the influxdb server.
    /// Calling this method erases all data extracted from previous calls the ``process_message``.
    /// #Arguments
    /// *message - The ``DigitizerAnalogTraceMessage`` instance from which the data is extracted
    /// #Returns
    /// An emtpy result or an error arrising a malformed ``DigitizerAnalogTraceMessage`` parameter.
    async fn process_message(&mut self, message: &DigitizerAnalogTraceMessage) -> Result<()> {
        if self.num_batches == self.batch_size {
            self.num_batches = 0;
            self.measurements.clear();    
        }
        self.num_batches += 1;
        // Obtain message data, and error check
        self.frame_data.init(message).unwrap();
        self.frame_data.extract_channel_data(self.num_channels, message)?;
        //test_channels(message,8).unwrap();  //  TODO influxdb is used then this should be implemented properly

        let table_name = self.frame_data.get_table_name();
        let channel_strings : Vec<_> = (0..self.num_channels)
            .map(|c|"c".to_owned() + &c.to_string())
            .collect();

        for i in 0..self.frame_data.num_samples {
            let mut query = WriteQuery::new(self.frame_data.calc_measurement_time(i).into(), &table_name);
            for (c,channel) in channel_strings.iter().enumerate() {
                query = query.add_field(channel, self.frame_data.trace_data[c][i]);
            }
            self.measurements.push(query);
        }
        Ok(())
    }
    /// Sends data extracted from a previous call to ``process_message`` to the influxdb server.
    /// #Returns
    /// A string result, or an error arrising from the influxdb queries.
    async fn post_message(&mut self) -> Result<usize> {
        if self.num_batches == self.batch_size {
            self.client.query(&self.measurements).await.unwrap();
            /*for chunk in self.measurements.as_slice().chunks(10000) {
                self.client.query(chunk.to_vec()).await.unwrap();
            }*/
        }
        Ok(0)
    }
}

#[cfg(test)]
mod test {
    use influxdb::ReadQuery;

    use super::*;
/*
    #[tokio::test]
    async fn test_create() {
        let influx_db: InfluxDBEngine = InfluxDBEngine::new().await;
        assert!(influx_db.client.ping().await.is_ok());
    }
    #[tokio::test]
    async fn test_database_name() {
        let influx_db: InfluxDBEngine = InfluxDBEngine::new().await;
        assert_eq!(influx_db.client.database_name(), influx_db.database);
    }

    #[tokio::test]
    async fn test_insert() {
        let influx_db: InfluxDBEngine = InfluxDBEngine::new().await;
        influx_db.reset_database().await.unwrap();
        let write_result: std::result::Result<String, influxdb::Error> = influx_db
            .client
            .query(Measurement::default().into_query("template"))
            .await;
        assert!(write_result.is_ok());
    }
    #[tokio::test]
    async fn test_query() {
        let influx_db: InfluxDBEngine = InfluxDBEngine::new().await;
        influx_db.reset_database().await.unwrap();
        let query = ReadQuery::new(format!("SELECT * from {}", influx_db.database));
        let read_result = influx_db.client.query(query).await;
        assert!(read_result.is_ok());
    }
    #[tokio::test]
    async fn test_insert_and_query() {
        let influx_db: InfluxDBEngine = InfluxDBEngine::new().await;
        influx_db.reset_database().await.unwrap();
        let write_result = influx_db
            .client
            .query(
                Measurement {
                    time: DateTime::<Utc>::from_utc(
                        chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
                            .unwrap()
                            .and_hms_nano_opt(2, 0, 0, 10_000)
                            .unwrap(),
                        Utc,
                    ),
                    digitizer_id: 4,
                    frame_number: 0,
                    channel: 6,
                    intensity: 23,
                }
                .into_query("template"),
            )
            .await;
        let query = ReadQuery::new("SELECT * from template WHERE time >= '2000-01-01 02:00:00'");
        assert!(write_result.is_ok());
        let read_result = influx_db.client.query(query).await;
        //assert!(read_result.is_ok());
        println!("{}", read_result.unwrap());
    } */
}
