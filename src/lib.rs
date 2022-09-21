//! ESP-IDF Partition Tables
//!
//! <https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/partition-tables.html>

pub use self::{
    error::Error,
    partition::{AppType, DataType, Flags, Partition, SubType, Type},
};

mod error;
mod partition;

/// A partition table
#[derive(Debug, Clone)]
pub struct PartitionTable {
    partitions: Vec<Partition>,
}

impl PartitionTable {
    /// Construct a new partition table from a vector of partitions
    pub fn new(partitions: Vec<Partition>) -> Self {
        Self { partitions }
    }

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
        } else if let Ok(part_table) =
            Self::try_from_str(String::from_utf8(input).map_err(|_| Error::InvalidPartitionTable)?)
        {
            Ok(part_table)
        } else {
            Err(Error::InvalidPartitionTable)
        }
    }

    /// Attempt to parse a binary partition table from the given bytes.
    ///
    /// For more information on the partition table format see:
    /// <https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/partition-tables.html>
    pub fn try_from_bytes<S>(_data: S) -> Result<Self, Error>
    where
        S: Into<Vec<u8>>,
    {
        todo!()
    }

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
            .has_headers(false)
            .trim(csv::Trim::All)
            .from_reader(data.as_bytes());

        let capacity = data.lines().count();
        let mut partitions = Vec::with_capacity(capacity);

        // Default offset is 0x8000 in ESP-IDF, partition table size is 0x1000
        let mut offset = 0x9000;

        for record in reader.deserialize() {
            // Since offsets are optional, we need to update the deserialized partition when
            // this field is omitted
            let mut partition: Partition = record?;
            offset = partition.fix_offset(offset);

            partitions.push(partition);
        }

        Ok(Self::new(partitions))
    }

    /// Return a reference to a vector containing each partition in the
    /// partition table
    pub fn partitions(&self) -> &Vec<Partition> {
        &self.partitions
    }

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
}
