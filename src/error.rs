/// Partition table error
#[derive(Debug)]
pub enum Error {
    /// Two or more partitions with the same name were found
    DuplicatePartitions,
    /// The checksum in the binary data does not match the computed value
    InvalidChecksum,
    /// The partition table is invalid
    InvalidPartitionTable,
    /// The length of the binary data is not a multiple of 32 (bits)
    LengthNotMultipleOf32,
    /// No partition of type 'app' was found in the partition table
    NoAppPartition,
    /// No ned marker was found in the binary data
    NoEndMarker,
    /// Two partitions are overlapping each other
    OverlappingPartitions,
    /// The partition is not correctly aligned
    UnalignedPartition,

    /// An error which originated in the `csv` package
    CsvError(csv::Error),
    /// An error which originated in the `deku` package
    DekuError(deku::DekuError),
    /// An error which originated in the `std::io` module
    IoError(std::io::Error),
}

impl From<csv::Error> for Error {
    fn from(e: csv::Error) -> Self {
        Error::CsvError(e)
    }
}

impl From<deku::DekuError> for Error {
    fn from(e: deku::DekuError) -> Self {
        Error::DekuError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}
