use std::env;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader, Read};

use chrono::prelude::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

type Tags = Map<String, Value>;

// A measurement struct
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "metric_type", rename_all = "lowercase")]
enum Point {
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

struct Measurement {
    point: Point,
    timestamp: i64,
}

// Format a map of key/value pairs into a string
fn format_map_iter<'a>(iter: impl Iterator<Item = (&'a String, &'a Value)>) -> String {
    iter.map(|(k, v)| format_value(k, v))
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>()
        .join(",")
}

/// Format a key/value pair into the line protocol format
fn format_value(key: &str, value: &Value) -> String {
    match value {
        Value::Object(obj) => format_map_iter(obj.iter()),
        Value::Bool(b) => format!("{key}={b}"),
        Value::Number(n) => format!("{key}={n}"),
        Value::String(s) => format!("{key}=\"{s}\""),
        Value::Null => "".to_string(),
        Value::Array(a) => a
            .iter()
            .enumerate()
            .map(|(i, v)| format_value(&format!("{key}_{i}"), v))
            .collect::<Vec<String>>()
            .join(","),
    }
}

/// Format a set of tags into a string
fn format_tags(tags: &Tags, extra_tags: Option<Tags>) -> String {
    format_map_iter(tags.iter().chain(extra_tags.unwrap_or_default().iter()))
}

impl Measurement {
    /// Convert a measurement to a line protocol string
    fn to_line_protocol(&self, extra_tags: Option<Tags>) -> String {
        match &self.point {
            Point::Timer {
                metric,
                value,
                tags,
            } => {
                format!(
                    "{},{} value={} {}",
                    metric,
                    format_tags(tags, extra_tags),
                    value,
                    self.timestamp
                )
            }
            Point::Counter {
                metric,
                value,
                tags,
            } => {
                format!(
                    "{},{} value={} {}",
                    metric,
                    format_tags(tags, extra_tags),
                    value,
                    self.timestamp
                )
            }
        }
    }

    /// Parse a line from a singer metric log
    fn from_singer_metric_line(line: &str) -> Result<Self, String> {
        let mut parts = line.splitn(2, " INFO METRIC: ");

        let datestr = parts.next().expect("No timestamp found in line");
        let datetime = Utc
            .datetime_from_str(datestr, "%Y-%m-%d %H:%M:%S,%f")
            .expect("Invalid timestamp found in line");

        let json_string = parts.next().expect("No measurement JSON found in line");
        let point: Point = serde_json::from_str(json_string).expect("Invalid JSON found in line");
        let measurement: Self = Self {
            point,
            timestamp: datetime.timestamp_nanos(),
        };
        Ok(measurement)
    }
}

fn process_buffer(buffer: BufReader<impl Read>) -> Result<(), String> {
    for line in buffer.lines() {
        let measurement = Measurement::from_singer_metric_line(&line.unwrap()).unwrap();
        println!("{}", measurement.to_line_protocol(None));
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // The second argument is the filename
    if let Some(filename) = args.get(1) {
        let file = File::open(filename).unwrap();

        let buffer = BufReader::new(file);
        process_buffer(buffer).unwrap();
    } else {
        // read from stdin
        let stdin = stdin();
        let buffer = BufReader::new(stdin);
        process_buffer(buffer).unwrap();
    }
}
