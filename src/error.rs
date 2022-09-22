/// Partition table error
#[derive(Debug)]
pub enum Error {
    /// The checksum in the binary data does not match the computed value
    InvalidChecksum,
    /// The partition table is invalid
    InvalidPartitionTable,
    /// The length of the binary data is not a multiple of 32 (bits)
    LengthNotMultipleOf32,
    /// No ned marker was found in the binary data
    NoEndMarker,

    /// An error which originated in the `csv` package
    CsvError(csv::Error),
    /// An error which originated in the `std::io` module
    IoError(std::io::Error),
}

impl From<csv::Error> for Error {
    fn from(e: csv::Error) -> Self {
        Error::CsvError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}
