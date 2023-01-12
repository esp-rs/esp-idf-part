#[cfg(not(feature = "std"))]
type String = heapless::String<{ crate::partition::MAX_NAME_LEN }>;
#[cfg(not(feature = "std"))]
type Vec<T> = heapless::Vec<T, { crate::MD5_NUM_MAGIC_BYTES }>;

/// Partition table error
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum Error {
    /// Two or more partitions with the same name were found
    #[cfg_attr(
        feature = "std",
        error("Two or more partitions with the same name ('{0}') were found")
    )]
    DuplicatePartitions(String),

    /// The checksum in the binary data does not match the computed value
    #[cfg_attr(
        feature = "std",
        error(
            "The binary's checksum is invalid (expected '{expected:?}', computed '{computed:?}')"
        )
    )]
    InvalidChecksum {
        expected: Vec<u8>,
        computed: Vec<u8>,
    },

    /// The partition table is invalid
    #[cfg_attr(feature = "std", error("The partition table is invalid"))]
    InvalidPartitionTable,

    /// The length of the binary data is not a multiple of 32
    #[cfg_attr(
        feature = "std",
        error("The length of the binary data is not a multiple of 32")
    )]
    LengthNotMultipleOf32,

    /// Multiple partitions with type 'app' and subtype 'factory' were found
    #[cfg_attr(
        feature = "std",
        error("Multiple partitions with type 'app' and subtype 'factory' were found")
    )]
    MultipleFactoryPartitions,

    /// No partition of type 'app' was found in the partition table
    #[cfg_attr(
        feature = "std",
        error("No partition of type 'app' was found in the partition table")
    )]
    NoAppPartition,

    /// No ned marker was found in the binary data
    #[cfg_attr(feature = "std", error("No ned marker was found in the binary data"))]
    NoEndMarker,

    /// Two partitions are overlapping each other
    #[cfg_attr(
        feature = "std",
        error("Two partitions are overlapping each other: '{0}' and '{1}'")
    )]
    OverlappingPartitions(String, String),

    /// The partition is not correctly aligned
    #[cfg_attr(feature = "std", error("The partition is not correctly aligned"))]
    UnalignedPartition,

    /// An error which originated in the `csv` package
    #[cfg(feature = "std")]
    #[cfg_attr(feature = "std", error(transparent))]
    CsvError(#[from] csv::Error),

    /// An error which originated in the `deku` package
    #[cfg(feature = "std")]
    #[cfg_attr(feature = "std", error(transparent))]
    DekuError(#[from] deku::DekuError),

    /// An error which occurred while trying to convert bytes to a String
    #[cfg(feature = "std")]
    #[cfg_attr(feature = "std", error(transparent))]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    /// An error which originated in the `std::io` module
    #[cfg(feature = "std")]
    #[cfg_attr(feature = "std", error(transparent))]
    IoError(#[from] std::io::Error),
}
