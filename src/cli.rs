use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::metric::Precision;

#[derive(Parser)]
#[command(author, version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// The input file to read from
    #[arg(short, long, value_name = "FILE")]
    pub input: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Convert Singer metrics to InfluxDB line protocol
    LineProtocol {
        /// The timestamp precision to use
        #[arg(short, long, value_name = "PRECISION", default_value = "ns")]
        precision: Precision,
    },
}
