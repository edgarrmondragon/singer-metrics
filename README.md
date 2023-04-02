# Singer Metrics Processing

This project is a collection of utilities for processing metrics from the Singer.io specification.

## Singer Metrics

The expected format of a Singer metric line is:

```
<TIMESTAMP> INFO METRIC: <METRIC JSON>
```

## Build

```sh
cargo build
```

### Python wheel

```sh
mature build
```

## InfluxDB line protocol

The `singer-metrics` CLI provides a `line-protocol` subcommand that can be used to convert Singer metrics to the [InfluxDB line protocol](https://docs.influxdata.com/influxdb/v2.6/reference/syntax/line-protocol/).

```console
$ echo 'INFO METRIC: {"metric_type": "timer", "metric": "http_request_duration", "value": 0.846369, "tags": {"my_tag": "abc"}}' \
  | singer-metrics line-protocol
http_request_duration,my_tag="abc" value=0.846369 1680468150224571000
```
