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
}
