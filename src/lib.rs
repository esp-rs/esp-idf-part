//! A library for parsing and generating ESP-IDF partition tables, both in the
//! binary and CSV formats as described in the ESP-IDF documentation.
//!
//! For additional information regarding the partition table format please refer
//! to the ESP-IDF documentation:  
//! <https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/partition-tables.html>

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "std")]
use core::ops::Rem;
#[cfg(feature = "std")]
use std::io::Write as _;

#[cfg(feature = "std")]
use deku::prelude::DekuContainerRead as _;

pub use self::{
    error::Error,
    partition::{AppType, DataType, Partition, SubType, Type},
};
#[cfg(feature = "std")]
use self::{
    hash_writer::HashWriter,
    partition::{DeserializedBinPartition, DeserializedCsvPartition, PARTITION_ALIGNMENT},
};

mod error;
mod partition;

#[cfg(not(feature = "std"))]
type Vec<T> = heapless::Vec<T, PARTITION_SIZE>;

#[cfg(feature = "std")]
const END_MARKER: [u8; 32] = [0xFF; 32];
#[cfg(feature = "std")]
const MAX_PARTITION_LENGTH: usize = 0xC00;
pub(crate) const MD5_NUM_MAGIC_BYTES: usize = 16;
#[cfg(feature = "std")]
const MD5_PART_MAGIC_BYTES: [u8; MD5_NUM_MAGIC_BYTES] = [
    0xEB, 0xEB, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
];
const PARTITION_SIZE: usize = 32;
#[cfg(feature = "std")]
const PARTITION_TABLE_SIZE: usize = 0x1000;

/// A partition table
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionTable {
    partitions: Vec<Partition>,
}

impl PartitionTable {
    /// Construct a new partition table from zero or more partitions
    ///
    /// Note that in order for a partition table to pass validation, it must
    /// have at least one partition with type [`Type::App`].
    pub fn new(partitions: Vec<Partition>) -> Self {
        Self { partitions }
    }

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    /// Attempt to parse either a binary or CSV partition table from the given
    /// input.
    ///
    /// For more information on the partition table format see:  
    /// <https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/partition-tables.html>
    pub fn try_from<S>(data: S) -> Result<Self, Error>
    where
        S: Into<Vec<u8>>,
    {
        let input: Vec<u8> = data.into();

        // If a partition table was detected from ESP-IDF (eg. using `esp-idf-sys`) then
        // it will be passed in its _binary_ form. Otherwise, it will be provided as a
        // CSV.
        if let Ok(part_table) = Self::try_from_bytes(&*input) {
            Ok(part_table)
        } else if let Ok(part_table) = Self::try_from_str(String::from_utf8(input)?) {
            Ok(part_table)
        } else {
            Err(Error::InvalidPartitionTable)
        }
    }

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    /// Attempt to parse a binary partition table from the given bytes.
    ///
    /// For more information on the partition table format see:  
    /// <https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/partition-tables.html>
    pub fn try_from_bytes<S>(data: S) -> Result<Self, Error>
    where
        S: Into<Vec<u8>>,
    {
        let data = data.into();

        // The data's MUST be an even multiple of 32
        if data.len() % 32 != 0 {
            return Err(Error::LengthNotMultipleOf32);
        }

        let mut ctx = md5::Context::new();

        let mut partitions = vec![];
        for line in data.chunks_exact(PARTITION_SIZE) {
            if line.starts_with(&MD5_PART_MAGIC_BYTES) {
                // The first 16 bytes are just the marker. The next 16 bytes is
                // the actual MD5 string.
                let digest_in_file = &line[16..32];
                let digest_computed = *ctx.clone().compute();

                if digest_computed != digest_in_file {
                    return Err(Error::InvalidChecksum {
                        expected: digest_in_file.to_vec(),
                        computed: digest_computed.to_vec(),
                    });
                }
            } else if line != END_MARKER {
                let (_, partition) = DeserializedBinPartition::from_bytes((line, 0))?;

                let partition = Partition::from(partition);
                partitions.push(partition);

                ctx.consume(line);
            } else {
                // We're finished parsing the binary data, time to construct and return the
                // [PartitionTable].
                let table = Self::new(partitions);
                table.validate()?;

                return Ok(table);
            }
        }

        Err(Error::NoEndMarker)
    }

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    /// Attempt to parse a CSV partition table from the given string.
    ///
    /// For more information on the partition table format see:  
    /// <https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/partition-tables.html>
    pub fn try_from_str<S>(data: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let data = data.into();
        let mut reader = csv::ReaderBuilder::new()
            .comment(Some(b'#'))
            .flexible(true)
            .has_headers(false)
            .trim(csv::Trim::All)
            .from_reader(data.as_bytes());

        // Default offset is 0x8000 in ESP-IDF, partition table size is 0x1000
        let mut offset = 0x9000;

        let mut partitions = vec![];
        for record in reader.deserialize() {
            // Since offsets are optional, we need to update the deserialized
            // partition when this field is omitted
            let mut partition: DeserializedCsvPartition = record?;
            offset = partition.fix_offset(offset);

            let partition = Partition::from(partition);
            partitions.push(partition);
        }

        let table = Self::new(partitions);
        table.validate()?;

        Ok(table)
    }

    /// Return a reference to a vector containing each partition in the
    /// partition table
    pub fn partitions(&self) -> &Vec<Partition> {
        &self.partitions
    }

    /// Find a partition with the given name in the partition table
    pub fn find(&self, name: &str) -> Option<&Partition> {
        self.partitions.iter().find(|p| p.name() == name)
    }

    /// Find a partition with the given type in the partition table
    pub fn find_by_type(&self, ty: Type) -> Option<&Partition> {
        self.partitions.iter().find(|p| p.ty() == ty)
    }

    /// Find a partition with the given type and subtype in the partition table
    pub fn find_by_subtype(&self, ty: Type, subtype: SubType) -> Option<&Partition> {
        self.partitions
            .iter()
            .find(|p| p.ty() == ty && p.subtype() == subtype)
    }

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    /// Convert a partition table to binary
    pub fn to_bin(&self) -> Result<Vec<u8>, Error> {
        let mut result = Vec::with_capacity(PARTITION_TABLE_SIZE);
        let mut hasher = HashWriter::new(&mut result);

        for partition in &self.partitions {
            partition.write_bin(&mut hasher)?;
        }

        let (writer, hash) = hasher.compute();

        writer.write_all(&MD5_PART_MAGIC_BYTES)?;
        writer.write_all(&hash.0)?;

        let written = self.partitions.len() * PARTITION_SIZE + 32;
        let padding = std::iter::repeat(0xFF)
            .take(MAX_PARTITION_LENGTH - written)
            .collect::<Vec<_>>();

        writer.write_all(&padding)?;

        Ok(result)
    }

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    /// Convert a partition table to a CSV string
    pub fn to_csv(&self) -> Result<String, Error> {
        let mut csv = String::new();

        // We will use the same common "header" that is used in ESP-IDF
        csv.push_str("# ESP-IDF Partition Table\n");
        csv.push_str("# Name,Type,SubType,Offset,Size,Flags\n");

        // Serialize each partition using a [csv::Writer]
        let mut writer = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(vec![]);

        for partition in &self.partitions {
            partition.write_csv(&mut writer)?;
        }

        // Append the serialized partitions to the header text, leaving us with our
        // completed CSV text
        csv.push_str(&String::from_utf8_lossy(&writer.into_inner().unwrap()));

        Ok(csv)
    }

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    /// Validate a partition table
    fn validate(&self) -> Result<(), Error> {
        // There must be at least one partition with type 'app'
        if self.find_by_type(Type::App).is_none() {
            return Err(Error::NoAppPartition);
        }

        // There can be at most one partition of type 'app' and of subtype 'factory'
        if self
            .partitions
            .iter()
            .filter(|p| p.ty() == Type::App && p.subtype() == SubType::App(AppType::Factory))
            .count()
            > 1
        {
            return Err(Error::MultipleFactoryPartitions);
        }

        for partition in &self.partitions {
            // Partitions of type 'app' have to be placed at offsets aligned to 0x10000
            // (64k)
            if partition.ty() == Type::App && partition.offset().rem(PARTITION_ALIGNMENT) != 0 {
                return Err(Error::UnalignedPartition);
            }
        }

        for partition_a in &self.partitions {
            for partition_b in &self.partitions {
                // Do not compare partitions with themselves :)
                if partition_a == partition_b {
                    continue;
                }

                // Partitions cannot have conflicting names
                if partition_a.name() == partition_b.name() {
                    return Err(Error::DuplicatePartitions(partition_a.name()));
                }

                // Partitions cannot overlap each other
                if partition_a.overlaps(partition_b) {
                    return Err(Error::OverlappingPartitions(
                        partition_a.name(),
                        partition_b.name(),
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(feature = "std")]
mod hash_writer {
    use md5::{Context, Digest};

    pub(crate) struct HashWriter<W> {
        inner: W,
        hasher: Context,
    }

    impl<W> std::io::Write for HashWriter<W>
    where
        W: std::io::Write,
    {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.hasher.write_all(buf)?;
            self.inner.write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.inner.flush()
        }
    }

    impl<W> HashWriter<W>
    where
        W: std::io::Write,
    {
        pub fn new(inner: W) -> Self {
            Self {
                inner,
                hasher: Context::new(),
            }
        }

        pub fn compute(self) -> (W, Digest) {
            (self.inner, self.hasher.compute())
        }
    }
}
