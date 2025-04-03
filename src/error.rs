/// Partition table errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Two or more partitions with the same name were found
    #[error("Two or more partitions with the same name ('{0}') were found")]
    DuplicatePartitions(String),

    /// The checksum in the binary data does not match the computed value
    #[error("The binary's checksum is invalid (expected '{expected:?}', computed '{computed:?}')")]
    InvalidChecksum {
        expected: Vec<u8>,
        computed: Vec<u8>,
    },

    /// Partition with type 'data' and subtype 'ota' must have size of 0x2000
    /// (8k) bytes
    #[error("Partition with type 'data' and subtype 'ota' must have size of 0x2000 (8k) bytes")]
    InvalidOtadataPartitionSize,

    /// The length of the binary data is not a multiple of 32
    #[error("The length of the binary data is not a multiple of 32")]
    LengthNotMultipleOf32,

    /// Multiple partitions with type 'app' and subtype 'factory' were found
    #[error("Multiple partitions with type 'app' and subtype 'factory' were found")]
    MultipleFactoryPartitions,

    /// Multiple partitions with type 'data' and subtype 'ota' were found
    #[error("Multiple partitions with type 'data' and subtype 'ota' were found")]
    MultipleOtadataPartitions,

    /// No partition of type 'app' was found in the partition table
    #[error("No partition of type 'app' was found in the partition table")]
    NoAppPartition,

    /// No ned marker was found in the binary data
    #[error("No ned marker was found in the binary data")]
    NoEndMarker,

    /// Two partitions are overlapping each other
    #[error("Two partitions are overlapping each other: '{0}' and '{1}'")]
    OverlappingPartitions(String, String),

    /// Partition is above the maximum supported size of 16MB
    #[error("Partition larger than maximum supported size of 16MB: '{0}'")]
    PartitionTooLarge(String),

    /// The partition is not correctly aligned
    #[error("The partition is not correctly aligned")]
    UnalignedPartition,

    /// An error which originated in the `csv` package
    #[error(transparent)]
    CsvError(#[from] csv::Error),

    /// An error which originated in the `deku` package
    #[error(transparent)]
    DekuError(#[from] deku::DekuError),

    /// An error which occurred while trying to convert bytes to a String
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    /// An error which originated in the `std::io` module
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
