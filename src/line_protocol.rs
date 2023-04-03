use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::metric::{Measurement, Point, Precision, Tags};
use crate::protocol_trait::ProtocolTrait;

/// InfluxDB line protocol
///
/// `https://docs.influxdata.com/influxdb/v2.6/reference/syntax/line-protocol/`
#[derive(Default)]
pub struct LineProtocol {
    /// An optional map of extra tags to add to each measurement
    pub extra_tags: Tags,

    /// The timestamp precision to use when formatting timestamps
    pub precision: Precision,
}

impl LineProtocol {
    /// Create a new LineProtocol instance
    /// # Arguments
    ///
    /// * `precision` - The precision to use when formatting timestamps
    /// * `extra_tags` - An optional map of extra tags to add to each
    ///   measurement
    ///
    /// # Returns
    ///
    /// A new LineProtocol instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use serde_json::Value;
    /// use singer_metrics::line_protocol::LineProtocol;
    /// use singer_metrics::metric::{Precision, Tags};
    /// use std::collections::HashMap;
    ///
    /// let mut extra_tags = Tags::new();
    /// extra_tags.insert("host".to_string(), Value::String("localhost".to_string()));
    ///
    /// let protocol = LineProtocol::new(Precision::default(), Some(extra_tags));
    ///
    /// assert!(matches!(protocol.precision, Precision::Nanoseconds));
    /// assert_eq!(protocol.extra_tags.len(), 1);
    /// ```
    pub fn new(precision: Precision, extra_tags: Option<Tags>) -> Self {
        LineProtocol {
            precision,
            extra_tags: extra_tags.unwrap_or_default(),
        }
    }
}

impl ProtocolTrait for LineProtocol {
    /// Dump a measurement to a string
    ///
    /// # Arguments
    ///
    /// * `measurement` - The measurement to dump
    ///
    /// # Returns
    ///
    /// A string representation of the measurement
    ///
    /// # Example
    ///
    /// ```rust
    /// use chrono::Utc;
    /// use serde_json::Value;
    /// use singer_metrics::line_protocol::LineProtocol;
    /// use singer_metrics::metric::{Measurement, Tags};
    /// use singer_metrics::protocol_trait::ProtocolTrait;
    ///
    /// let protocol = LineProtocol::default();
    ///
    /// let line = protocol.dump(&Measurement {
    ///    point: singer_metrics::metric::Point::Timer {
    ///       metric: "test".to_string(),
    ///       value: 1.0,
    ///       tags: Tags::from_iter(vec![("tag1".to_string(), Value::String("value1".to_string()))]),
    ///    },
    ///    timestamp: Utc::now(),
    /// });
    ///
    /// assert!(line.starts_with("test,"));
    fn dump(&self, measurement: &Measurement) -> String {
        match &measurement.point {
            Point::Timer {
                metric,
                value,
                tags,
            } => {
                format!(
                    "{}{} value={} {}",
                    metric,
                    format_tags(tags, &self.extra_tags),
                    value,
                    format_datetime(&measurement.timestamp, &self.precision),
                )
            }
            Point::Counter {
                metric,
                value,
                tags,
            } => {
                format!(
                    "{}{} value={} {}",
                    metric,
                    format_tags(tags, &self.extra_tags),
                    value,
                    format_datetime(&measurement.timestamp, &self.precision),
                )
            }
        }
    }
}

fn _add_key_prefix(key: &str, prefix: Option<&str>) -> String {
    if let Some(prefix) = prefix {
        return format!("{}__{}", prefix, key);
    }
    key.to_string()
}

// Format a map of key/value pairs into a string
fn format_map_iter<'a>(
    iter: impl Iterator<Item = (&'a String, &'a Value)>,
    prefix: Option<&str>,
) -> String {
    iter.map(|(k, v)| (_add_key_prefix(k, prefix), v))
        .map(|(k, v)| format_tag_value(&k, v))
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>()
        .join(",")
}

/// Format a key/value pair into the line protocol format
fn format_tag_value(key: &str, value: &Value) -> String {
    // Escape whitespace in key
    let key = key.replace(' ', "\\ ");

    match value {
        Value::Object(obj) => format_map_iter(obj.iter(), Some(&key)),
        Value::Bool(b) => format!("{key}={b}"),
        Value::Number(n) => format!("{key}={n}"),
        Value::String(s) => {
            let s = s.replace(' ', "\\ ");
            format!("{key}={s}")
        }
        // Value::String(s) => format!("{key}=\"{s}\""),
        Value::Null => "".to_string(),
        Value::Array(a) => a
            .iter()
            .enumerate()
            .map(|(i, v)| format_tag_value(&format!("{key}_{i}"), v))
            .collect::<Vec<String>>()
            .join(","),
    }
}

/// Format a set of tags into a string
fn format_tags(tags: &Tags, extra_tags: &Tags) -> String {
    let mut tags_string = format_map_iter(tags.iter().chain(extra_tags.iter()), None);
    if !tags_string.is_empty() {
        tags_string.insert(0, ',');
    }
    tags_string
}

/// Convert a datetime to a string in the specified precision
fn format_datetime(dt: &DateTime<Utc>, precision: &Precision) -> String {
    match precision {
        Precision::Nanoseconds => dt.timestamp_nanos().to_string(),
        Precision::Microseconds => dt.timestamp_millis().to_string(),
        Precision::Milliseconds => dt.timestamp_millis().to_string(),
        Precision::Seconds => dt.timestamp().to_string(),
    }
}

#[test]
fn test_line_protocol() {
    use chrono::Utc;
    use serde_json::Value;

    let protocol = LineProtocol::default();

    let line = protocol.dump(&Measurement {
        point: Point::Timer {
            metric: "test".to_string(),
            value: 1.23,
            tags: Tags::from_iter(vec![
                ("tag1".to_string(), Value::String("value1".to_string())),
                ("tag2".to_string(), Value::Number(2.into())),
            ]),
        },
        timestamp: DateTime::parse_from_rfc3339("2021-06-25T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc),
    });

    assert_eq!(
        line,
        "test,tag1=value1,tag2=2 value=1.23 1624579200000000000"
    );
}

#[test]
fn test_line_protocol_empty_tags() {
    use chrono::Utc;

    let protocol = LineProtocol::default();

    let line = protocol.dump(&Measurement {
        point: Point::Timer {
            metric: "test".to_string(),
            value: 1.23,
            tags: Tags::new(),
        },
        timestamp: DateTime::parse_from_rfc3339("2021-06-25T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc),
    });

    assert_eq!(line, "test value=1.23 1624579200000000000");
}

#[test]
fn test_line_protocol_ms_precision() {
    use chrono::Utc;

    let protocol = LineProtocol::new(Precision::Milliseconds, None);

    let line = protocol.dump(&Measurement {
        point: Point::Timer {
            metric: "test".to_string(),
            value: 1.23,
            tags: Tags::new(),
        },
        timestamp: DateTime::parse_from_rfc3339("2021-06-25T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc),
    });

    assert_eq!(line, "test value=1.23 1624579200000");
}

#[test]
fn test_line_protocol_extra_tags() {
    use chrono::Utc;
    use serde_json::Value;

    let protocol = LineProtocol::new(
        Precision::Seconds,
        Some(Tags::from_iter(vec![
            ("tag3".to_string(), Value::String("value3".to_string())),
            ("tag4".to_string(), Value::Number(4.into())),
        ])),
    );

    let line = protocol.dump(&Measurement {
        point: Point::Timer {
            metric: "test".to_string(),
            value: 1.23,
            tags: Tags::from_iter(vec![
                ("tag1".to_string(), Value::String("value1".to_string())),
                ("tag2".to_string(), Value::Number(2.into())),
            ]),
        },
        timestamp: DateTime::parse_from_rfc3339("2021-06-25T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc),
    });

    assert_eq!(
        line,
        "test,tag1=value1,tag2=2,tag3=value3,tag4=4 value=1.23 1624579200"
    );
}

#[test]
fn test_line_protocol_nested_tags() {
    use chrono::Utc;
    use serde_json::Value;

    let protocol = LineProtocol::default();

    let line = protocol.dump(&Measurement {
        point: Point::Timer {
            metric: "test".to_string(),
            value: 1.23,
            tags: Tags::from_iter(vec![
                ("tag1".to_string(), Value::String("value1".to_string())),
                ("tag2".to_string(), Value::Number(2.into())),
                (
                    "tag3".to_string(),
                    Value::Object(
                        Tags::from_iter(vec![
                            ("tag4".to_string(), Value::String("value4".to_string())),
                            ("tag5".to_string(), Value::Number(5.into())),
                        ])
                        .into(),
                    ),
                ),
            ]),
        },
        timestamp: DateTime::parse_from_rfc3339("2021-06-25T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc),
    });

    assert_eq!(
        line,
        "test,tag1=value1,tag2=2,tag3__tag4=value4,tag3__tag5=5 value=1.23 1624579200000000000"
    );
}

#[test]
fn test_escape_whitespace() {
    use chrono::Utc;

    let protocol = LineProtocol::new(Precision::Milliseconds, None);

    let line = protocol.dump(&Measurement {
        point: Point::Counter {
            metric: "test".to_string(),
            value: 42,
            tags: Tags::from_iter(vec![
                (
                    "Tag With Spaces".to_string(),
                    Value::String("ValueNoSpaces".to_string()),
                ),
                (
                    "TagNoSpaces".to_string(),
                    Value::String("Value With Spaces".to_string()),
                ),
            ]),
        },
        timestamp: DateTime::parse_from_rfc3339("2021-06-25T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc),
    });

    assert_eq!(
        line,
        "test,Tag\\ With\\ Spaces=ValueNoSpaces,TagNoSpaces=Value\\ With\\ Spaces value=42 \
         1624579200000"
    );
}
