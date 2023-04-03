use std::io::{BufReader, BufWriter, Read, Write};

use crate::metric::Measurement;

pub trait ProtocolTrait {
    /// Dump a measurement to a string
    ///
    /// # Arguments
    ///
    /// * `measurement` - The measurement to dump
    ///
    /// # Returns
    ///
    /// A string representation of the measurement
    fn dump(&self, measurement: &Measurement) -> String;

    /// Convert input
    ///
    /// # Arguments
    ///
    /// * `reader` - The reader to read from
    /// * `writer` - The writer to write to
    fn convert(&self, reader: BufReader<impl Read>, mut writer: BufWriter<impl Write>) {
        Measurement::read(reader)
            .map(|measurement| self.dump(&measurement.unwrap()))
            .for_each(|line| writeln!(&mut writer, "{}", line).unwrap());
    }

    /// Convert input from reader to stdout
    ///
    /// # Arguments
    ///
    /// * `reader` - The reader to read from
    fn to_stdout(&self, reader: BufReader<impl Read>) {
        let stdout = std::io::stdout();
        let writer = BufWriter::new(stdout.lock());
        self.convert(reader, writer)
    }

    /// Pipe from stdin to stdout
    fn pipe(&self) {
        let stdin = std::io::stdin();
        self.to_stdout(BufReader::new(stdin.lock()));
    }
}
