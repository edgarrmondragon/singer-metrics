use std::io::{BufRead, BufReader, Read};

use chrono::prelude::{TimeZone, Utc};
use chrono::DateTime;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub type Tags = Map<String, Value>;

lazy_static! {
    static ref SINGER_METRIC_PATTERN: Regex =
        Regex::new(r"^(?P<timestamp>.+?)?\s*?INFO METRIC: (?P<metric_json>.*)$").unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "metric_type", rename_all = "lowercase")]
pub enum Point {
    Timer {
        metric: String,
        value: f64,
        tags: Tags,
    },
    Counter {
        metric: String,
        value: i64,
        tags: Tags,
    },
}

#[derive(Debug, Default)]
pub enum Precision {
    #[default]
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
}

impl Precision {
    pub fn from_string(s: &str) -> Result<Self, String> {
        match s {
            "ns" => Ok(Precision::Nanoseconds),
            "us" => Ok(Precision::Microseconds),
            "ms" => Ok(Precision::Milliseconds),
            "s" => Ok(Precision::Seconds),
            _ => Err(format!("Invalid precision: {}", s)),
        }
    }
}

pub struct Measurement {
    pub point: Point,
    pub timestamp: DateTime<Utc>,
}

impl Measurement {
    /// Parse a line from a singer metric log
    fn from_singer_metric_line(line: &str) -> Result<Self, String> {
        let caps = SINGER_METRIC_PATTERN
            .captures(line)
            .ok_or_else(|| format!("Invalid line: {}", line))?;

        let timestamp = caps.name("timestamp").map_or(Ok(Utc::now()), |ts| {
            Utc.datetime_from_str(ts.as_str(), "%Y-%m-%d %H:%M:%S,%f")
                .map_err(|e| format!("Invalid timestamp: {}", e))
        })?;

        let json_string = caps
            .name("metric_json")
            .ok_or_else(|| format!("No measurement JSON found in line: {}", line))?
            .as_str();
        let point: Point = serde_json::from_str(json_string).expect("Invalid JSON found in line");

        let measurement: Self = Self { point, timestamp };
        Ok(measurement)
    }

    /// Read a file of singer metric lines into an iterator of measurements
    pub fn read(buffer: BufReader<impl Read>) -> impl Iterator<Item = Result<Self, String>> {
        buffer
            .lines()
            .map(|line| Self::from_singer_metric_line(&line.unwrap()))
    }
}

#[test]
fn test_from_singer_metric_line() {
    let line = "2020-10-01 00:00:00,000 INFO METRIC: {\"metric_type\": \"timer\", \"metric\": \"test\", \"value\": 1.0, \"tags\": {\"tag1\": \"value1\"}}";
    let measurement = Measurement::from_singer_metric_line(line).unwrap();

    assert_eq!(
        measurement
            .timestamp
            .format("%Y-%m-%d %H:%M:%S,%f")
            .to_string(),
        "2020-10-01 00:00:00,000000000"
    );
    assert!(
        matches!(measurement.point, Point::Timer { metric, value, tags } if metric == "test" && value == 1.0 && tags.len() == 1)
    );
}

#[test]
fn test_from_singer_metric_line_no_timestamp() {
    let line = "INFO METRIC: {\"metric_type\": \"timer\", \"metric\": \"test\", \"value\": 1.0, \"tags\": {\"tag1\": \"value1\"}}";
    let measurement = Measurement::from_singer_metric_line(line).unwrap();

    assert!(measurement.timestamp > Utc::now() - chrono::Duration::seconds(1));
    assert!(
        matches!(measurement.point, Point::Timer { metric, value, tags } if metric == "test" && value == 1.0 && tags.len() == 1)
    );
}
