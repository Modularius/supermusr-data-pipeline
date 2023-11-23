use anyhow::{Error, Result};
use async_trait::async_trait;

use itertools::Itertools;
use log::debug;
use taos::{AsyncBindable, AsyncQueryable, AsyncTBuilder, Stmt, Taos, TaosBuilder, Value, ColumnView};

use streaming_types::dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage;

use super::{
    error_reporter::TDEngineErrorReporter,
    framedata::FrameData,
    tdengine_views::{create_column_views, create_frame_column_views},
    StatementErrorCode, TDEngineError, TimeSeriesEngine,
};

pub(crate) struct TDEngine {
    client: Taos,
    database: String,
    stmt: Stmt,
    frame_stmt: Stmt,
    error: TDEngineErrorReporter,
    num_channels: usize,
    frame_data: Vec<FrameData>,
    batch_size: usize,
    num_batches: usize,
    column_views: Vec<ColumnView>,
}

impl TDEngine {
    pub(crate) async fn new(
        broker: String,
        username: Option<String>,
        password: Option<String>,
        database: String,
        num_channels: usize,
        batch_size: usize,
        ws: bool
    ) -> Result<Self, Error> {
        let protocol = if ws {
            "taos+ws"
        } else {
            "taos"
        };
        let url = match Option::zip(username, password) {
            Some((username, password)) => format!("{protocol}://{broker}@{username}:{password}"),
            None => format!("{protocol}://{broker}"),
        };

        debug!("Creating TaosBuilder with url {url}");
        let client = TaosBuilder::from_dsn(url)
            .map_err(TDEngineError::TaosBuilder)?
            .build()
            .await
            .map_err(TDEngineError::TaosBuilder)?;

        let stmt = Stmt::init(&client)
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::Init, e))?;

        let frame_stmt = Stmt::init(&client)
            .await
            .map_err(TDEngineError::TaosBuilder)?;

        Ok(TDEngine {
            client,
            database,
            stmt,
            frame_stmt,
            error: TDEngineErrorReporter::new(),
            frame_data: Vec::new(),
            num_channels,
            batch_size,
            num_batches: usize::default(),
            column_views: Vec::new(),
        })
    }

    pub(crate) async fn create_database(&self) -> Result<(), TDEngineError> {
        self.client
            .exec(&format!(
                "CREATE DATABASE IF NOT EXISTS {} PRECISION 'ns'",
                self.database
            ))
            .await
            .map_err(TDEngineError::TaosBuilder)?;

        self.client
            .use_database(&self.database)
            .await
            .map_err(TDEngineError::TaosBuilder)
    }

    pub(crate) async fn init(
        &mut self,
    ) -> Result<(), TDEngineError> {
        self.create_supertable().await?;

        let template_table = self.database.to_owned() + ".template";
        let stmt_sql = format!(
            "INSERT INTO ? USING {template_table} TAGS (?) VALUES (?, ?{0})",
            ", ?".repeat(self.num_channels)
        );

        self.stmt
            .prepare(&stmt_sql)
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::Prepare, e))?;

        let frame_template_table = self.database.to_owned() + ".frame_template";
        let frame_stmt_sql = format!(
            "INSERT INTO ? USING {frame_template_table} TAGS (?) VALUES (?, ?, ?, ?, ?{0})",
            ", ?".repeat(self.num_channels)
        );

        self.frame_stmt
            .prepare(&frame_stmt_sql)
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::Prepare, e))?;
        Ok(())
    }

    async fn create_supertable(&self) -> Result<(), TDEngineError> {
        let metrics_string = format!(
            "ts TIMESTAMP, frametime TIMESTAMP{0}",
            (0..self.num_channels)
                .map(|ch| format!(", c{ch} SMALLINT UNSIGNED"))
                .fold(String::new(), |a, b| a + &b)
        );
        let template_table = self.database.to_owned() + ".template";
        let string = format!("CREATE STABLE IF NOT EXISTS {template_table} ({metrics_string}) TAGS (digitizer_id TINYINT UNSIGNED)");
        self.client
            .exec(&string)
            .await
            .map_err(|e| TDEngineError::SqlError(string.clone(), e))?;

        let frame_metrics_string = format!("frame_ts TIMESTAMP, sample_count INT UNSIGNED, sampling_rate INT UNSIGNED, frame_number INT UNSIGNED, error_code INT UNSIGNED{0}",
            (0..self.num_channels)
                .map(|ch|format!(", cid{ch} INT UNSIGNED"))
                .fold(String::new(),|a,b|a + &b)
        );
        let frame_template_table = self.database.to_owned() + ".frame_template";
        let string = format!("CREATE STABLE IF NOT EXISTS {frame_template_table} ({frame_metrics_string}) TAGS (digitizer_id TINYINT UNSIGNED)");
        self.client
            .exec(&string)
            .await
            .map_err(|e| TDEngineError::SqlError(string.clone(), e))?;
        Ok(())
    }

    pub(crate) async fn bind_samples(&mut self) -> Result<()> {
        self.stmt
            .bind(&self.column_views)
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::Bind, e))
            .unwrap();
        Ok(())
    }
}

#[async_trait]
impl TimeSeriesEngine for TDEngine {
    /// Takes a reference to a ``DigitizerAnalogTraceMessage`` instance and extracts the relevant data from it.
    /// The user should then call ``post_message`` to send the data to the tdengine server.
    /// Calling this method erases all data extracted from previous calls the ``process_message``.
    /// #Arguments
    /// *message - The ``DigitizerAnalogTraceMessage`` instance from which the data is extracted
    /// #Returns
    /// An emtpy result or an error arrising a malformed ``DigitizerAnalogTraceMessage`` parameter.
    async fn process_message(&mut self, message: &DigitizerAnalogTraceMessage) -> Result<()> {
        // Obtain the channel data, and error check
        self.error.test_metadata(message);

        // Obtain message data, and error check
        let mut frame_data = FrameData::default();
        frame_data.init(message)?;

        // Obtain the channel data, and error check
        self.error
            .test_channels(&frame_data, &message.channels().unwrap());

        frame_data.extract_channel_data(self.num_channels, message)?;

        self.frame_data.clear();
        self.frame_data.push(frame_data);
        self.num_batches += 1;
        let mut table_name = self.frame_data[0].get_table_name();

        let mut frame_table_name = self.frame_data[0].get_frame_table_name();
        frame_table_name.insert(0, '.');
        frame_table_name.insert_str(0, &self.database);
        table_name.insert(0, '.');
        table_name.insert_str(0, &self.database);



        // collate views
        let frame_column_views = create_frame_column_views(self.num_channels, self.frame_data.as_slice(), &self.error)?;
        self.column_views = create_column_views(self.num_channels, self.frame_data.as_slice())?;
        let tags = [Value::UTinyInt(self.frame_data[0].digitizer_id)];

        // Put this in another method
        //  Initialise Statement
        self.stmt
            .set_tbname(&table_name)
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::SetTableName, e))
            .unwrap();
        self.stmt
            .set_tags(&tags)
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::SetTags, e))
            .unwrap();

        self.frame_stmt
            .set_tbname(&frame_table_name)
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::SetTableName, e))
            .unwrap();
        self.frame_stmt
            .set_tags(&tags)
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::SetTags, e))
            .unwrap();
        
        self.frame_stmt
            .bind(&frame_column_views)
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::Bind, e))
            .unwrap();
        self.frame_stmt
            .add_batch()
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::AddBatch, e))
            .unwrap();
        Ok(())
    }

    /// Sends data extracted from a previous call to ``process_message`` to the tdengine server.
    /// #Returns
    /// The number of rows affected by the post or an error
    async fn post_message(&mut self) -> Result<usize> {
        self.stmt
            .add_batch()
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::AddBatch, e))
            .unwrap();

        //println!("{0}",self.frame_data.len());
        if self.num_batches < self.batch_size {
            return Ok(0)
        }
        self.num_batches = usize::default();


        let result = self
            .stmt
            .execute()
            .await
            .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::Execute, e))
            .unwrap()
            + self
                .frame_stmt
                .execute()
                .await
                .map_err(|e| TDEngineError::TaosStmt(StatementErrorCode::Execute, e))?;

        Ok(result)
    }
}













use std::default;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{Channel, DigitizerId, FrameNumber, Intensity};
use influxdb::{Client, InfluxDbWriteable, ReadQuery, WriteQuery};
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
}

impl InfluxDBEngine {
    /// Creates a new instance of InfluxDBEngine
    /// #Returns
    /// An instance connected to "http://localhost:8086" and database "TraceLogs".
    /// The token used to authenticate with the influxdb server is currently hardcoded.
    pub async fn new() -> Self {
        let url = dotenv::var("INFLUXDB_URL")
            .unwrap_or_else(|e| panic!("INFLUXDB_URL not found in .env: {}", e));
        let database = dotenv::var("INFLUXDB_DATABASE")
            .unwrap_or_else(|e| panic!("INFLUXDB_DATABASE not found in .env: {}", e));
        let token = dotenv::var("INFLUXDB_TOKEN")
            .unwrap_or_else(|e| panic!("INFLUXDB_TOKEN not found in .env: {}", e));
        InfluxDBEngine {
            url: url.clone(),
            database: database.clone(),
            client: Client::new(url, database).with_token(token), /*with_auth("admin", "password"),*/
            frame_data: FrameData::default(),
            measurements: Vec::<WriteQuery>::default(),
        }
    }

    /// Clears all data from database "TraceLogs" and resets it.
    /// #Returns
    /// An emtpy result or an error arrising from the influxdb queries.
    pub async fn reset_database(&self) -> Result<()> {
        self.client
            .query(ReadQuery::new(format!("DROP DATABASE {}", self.database)))
            .await?;
        self.client
            .query(ReadQuery::new(format!("CREATE DATABASE {}", self.database)))
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
        self.measurements.clear();
        // Obtain message data, and error check
        self.frame_data.init(message)?;
        //test_channels(message,8).unwrap();  //  TODO influxdb is used then this should be implemented properly

        for channel in message.channels().unwrap() {
            for (i, v) in channel.voltage().iter().flatten().enumerate() {
                self.measurements.push(
                    self.frame_data
                        .make_measurement(channel.channel(), i, v)
                        .into_query("template"),
                );
            }
        }
        Ok(())
    }
    /// Sends data extracted from a previous call to ``process_message`` to the influxdb server.
    /// #Returns
    /// A string result, or an error arrising from the influxdb queries.
    async fn post_message(&mut self) -> Result<String> {
        self.client.query(&self.measurements).await?;
        Ok("".to_owned())
    }
}

#[cfg(test)]
mod test {
    use influxdb::ReadQuery;

    use super::*;

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
    }
}
