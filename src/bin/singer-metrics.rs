use std::fs::File;
use std::io::BufReader;

use clap::Parser;
use singer_metrics::cli::{Cli, Commands};
use singer_metrics::line_protocol::LineProtocol;
use singer_metrics::protocol_trait::ProtocolTrait;

fn main() {
    let cli = Cli::parse();

    let protocol = match cli.command {
        Commands::LineProtocol { precision } => LineProtocol::new(precision, None),
    };

    if let Some(filename) = cli.input {
        // read from file
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        protocol.to_stdout(reader);
    } else {
        // read from stdin
        protocol.pipe();
    }
}
