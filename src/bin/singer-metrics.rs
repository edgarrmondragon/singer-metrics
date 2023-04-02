use std::fs::File;
use std::io::{stdin, BufReader};
use std::path::PathBuf;

use clap::Parser;
use singer_metrics::cli::{Cli, Commands};
use singer_metrics::line_protocol::LineProtocol;
use singer_metrics::metric::{Measurement, Precision};
use singer_metrics::protocol_trait::ProtocolTrait;

fn convert_to_line_protocol(input: Option<PathBuf>, precision: Precision) {
    let protocol = LineProtocol::new(precision, None);

    if let Some(filename) = input {
        // read from file
        let file = File::open(filename).unwrap();
        let buffer = BufReader::new(file);

        Measurement::read(buffer)
            .map(|measurement| protocol.dump(&measurement.unwrap()))
            .for_each(|line| println!("{}", line));
    } else {
        // read from stdin
        let stdin = stdin();
        let buffer = BufReader::new(stdin);

        Measurement::read(buffer)
            .map(|measurement| protocol.dump(&measurement.unwrap()))
            .for_each(|line| println!("{}", line));
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::LineProtocol { input, precision } => {
            let precision = Precision::from_string(&precision).expect("Invalid precision");
            convert_to_line_protocol(input, precision);
        }
    };
}
