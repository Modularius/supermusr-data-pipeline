use super::DigitiserDataHashMap;
use chrono::{DateTime, Timelike, Utc};
use ratatui::widgets::TableState;

/// Holds the current state of the program.
pub struct App {
    pub table_headers: Vec<String>,
    pub table_body: Vec<Vec<String>>,
    pub table_state: TableState,
}

impl App {
    /// Create a new instance with default values.
    pub fn new(table_headers: &[&str]) -> App {
        App {
            table_headers: table_headers
            .iter()
            .map(|s| s.to_string())
            .collect(),
            table_body: Vec::new(),
            table_state: TableState::default(),
        }
    }

    pub fn generate_table_body(&mut self, common_dig_data_map: DigitiserDataHashMap) {
        // Clear table body.
        self.table_body.clear();
        let logged_data = common_dig_data_map
            .lock()
            .expect("should be able to lock common data");
        // Sort by digitiser ID.
        let mut sorted_data: Vec<_> = logged_data.iter().collect();
        sorted_data.sort_by_key(|x| x.0);
        // Add rows to table.
        for (digitiser_id, digitiser_data) in sorted_data.iter() {
            self.table_body.push(vec![
                // 1. Digitiser ID.
                digitiser_id.to_string(),
                // 2. Number of messages received.
                format!("{}", digitiser_data.msg_count),
                // 3. First message timestamp.
                format_timestamp(digitiser_data.first_msg_timestamp),
                // 4. Last message timestamp.
                format_timestamp(digitiser_data.last_msg_timestamp),
                // 5. Last message frame.
                format!("{}", digitiser_data.last_msg_frame),
                // 6. Message rate.
                format!("{:.1}", digitiser_data.msg_rate),
                // 7. Number of Bad Frames
                format!("{}", digitiser_data.bad_frame_count),
            ])
        }
    }

    pub fn generate_table_body2(&mut self, common_dig_data_map: DigitiserDataHashMap) {
        // Clear table body.
        self.table_body.clear();
        let logged_data = common_dig_data_map
            .lock()
            .expect("should be able to lock common data");
        // Sort by digitiser ID.
        let mut sorted_data: Vec<_> = logged_data.iter().collect();
        sorted_data.sort_by_key(|x| x.0);
        // Add rows to table.
        for (digitiser_id, digitiser_data) in sorted_data.iter() {
            self.table_body.push(vec![
                // 7. Number of channels present.
                format!("{}", digitiser_data.num_channels_present),
                // 8. Has the number of channels changed?
                format!(
                    "{}",
                    match digitiser_data.has_num_channels_changed {
                        true => "Yes",
                        false => "No",
                    }
                ),
                // 9. Number of samples in the first channel.
                format!("{}", digitiser_data.num_samples_in_first_channel),
                // 10. Is the number of samples identical?
                format!(
                    "{}",
                    match digitiser_data.is_num_samples_identical {
                        true => "Yes",
                        false => "No",
                    }
                ),
                // 11. Has the number of samples changed?
                format!(
                    "{}",
                    match digitiser_data.has_num_samples_changed {
                        true => "Yes",
                        false => "No",
                    }
                ),
                // 12. Number of Bad Frames
                format!("{}", digitiser_data.bad_frame_count),
                // 13. Min mean value
                format!("{:.2}", digitiser_data.mean_value.min().unwrap_or(-1.0)),
                // 14. Max mean value
                format!("{:.2}", digitiser_data.mean_value.max().unwrap_or(-1.0)),
                // 15. Max mean value
                digitiser_data.channels_present
                    .as_ref()
                    .map(|channels_present|channels_present.iter().fold(String::default(),|a, b|format!("{a} {b}")))
                    .unwrap_or_default()
            ])
        }
    }

    /// Move to the next item in the table.
    pub fn next(&mut self) {
        if self.table_body.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.table_body.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// Move to the previous item in the table.
    pub fn previous(&mut self) {
        if self.table_body.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.table_body.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
}

/// Create a neatly formatted String from a timestamp.
fn format_timestamp(timestamp: Option<DateTime<Utc>>) -> String {
    match timestamp {
        None => "N/A".to_string(),
        Some(t) => format!(
            "{}\n{}\n{} ns",
            t.format("%d/%m/%y"),
            t.format("%H:%M:%S"),
            t.nanosecond(),
        ),
    }
}
