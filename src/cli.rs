use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Convert Singer metrics to InfluxDB line protocol
    LineProtocol {
        /// The input file to read from
        #[arg(short, long, value_name = "FILE")]
        input: Option<PathBuf>,

        /// The timestamp precision to use
        #[arg(short, long, default_value = "ns")]
        precision: String,
    },
}
