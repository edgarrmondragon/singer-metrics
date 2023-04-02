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

The `singer_metrics` package provides a `singer-metrics-line-protocol` command line utility that can be used to convert Singer metrics to InfluxDB line protocol.

```sh
cat metrics.log | singer-metrics line-protocol > line_protocol.txt
```
