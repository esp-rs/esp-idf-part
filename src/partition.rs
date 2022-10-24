use core::fmt::Display;
use std::{
    cmp::{max, min},
    io::Write,
    str::FromStr,
};

use deku::{DekuContainerRead, DekuEnumExt, DekuError, DekuRead};
use regex::Regex;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use strum::{EnumIter, EnumString, EnumVariantNames, FromRepr, IntoEnumIterator};

const MAGIC_BYTES: [u8; 2] = [0xAA, 0x50];
const MAX_NAME_LEN: usize = 16;
pub(crate) const PARTITION_ALIGNMENT: u32 = 0x10000;

/// Supported partition types
///
/// User-defined partition types are allowed as long as their type ID does not
/// confict with [`Type::App`] or [`Type::Data`].
///
/// For additional information regarding the supported partition types, please
/// refer to the ESP-IDF documentation:  
/// <https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/partition-tables.html#type-field>
#[derive(Debug, Clone, Copy, PartialEq, Eq, DekuRead, Deserialize, Serialize)]
#[deku(endian = "little", type = "u8")]
#[serde(rename_all = "lowercase")]
pub enum Type {
    #[deku(id = "0x00")]
    App,
    #[deku(id = "0x01")]
    Data,
    #[deku(id_pat = "0x02..=0xFE")]
    Custom(u8),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Type::App | Type::Data => serde_plain::to_string(self).unwrap(),
                Type::Custom(ty) => format!("{:#04x}", ty),
            }
        )
    }
}

impl From<u8> for Type {
    fn from(ty: u8) -> Self {
        match ty {
            0x00 => Type::App,
            0x01 => Type::Data,
            ty => Type::Custom(ty),
        }
    }
}

impl Type {
    /// Return the numeric partition type ID for the given type
    pub fn as_u8(&self) -> u8 {
        match self {
            Type::App => 0x00,
            Type::Data => 0x01,
            Type::Custom(ty) => *ty,
        }
    }

    /// Return a `String` stating which subtypes are allowed for the given type.
    ///
    /// This is useful for error handling in dependent packages.
    pub fn subtype_hint(&self) -> String {
        match self {
            Type::App => "'factory', 'ota_0' through 'ota_15', or 'test'".into(),
            Type::Data => {
                let types = DataType::iter()
                    .map(|dt| format!("'{}'", serde_plain::to_string(&dt).unwrap()))
                    .collect::<Vec<_>>();

                let (tail, head) = types.split_last().unwrap();

                format!("{}, and {}", head.join(", "), tail)
            }
            Type::Custom(..) => "0x02 through 0xFE".into(),
        }
    }
}

/// Supported partition subtypes
///
/// User-defined partition subtypes are allowed as long as the partitions `Type`
/// is [`Type::Custom`].
///
/// For additional information regarding the supported partition subtypes,
/// please refer to the ESP-IDF documentation:  
/// <https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/partition-tables.html#subtype>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SubType {
    App(AppType),
    Data(DataType),
    Custom(u8),
}

impl Display for SubType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SubType::App(ty) => serde_plain::to_string(ty).unwrap(),
                SubType::Data(ty) => serde_plain::to_string(ty).unwrap(),
                SubType::Custom(ty) => format!("{:#04x}", ty),
            }
        )
    }
}

impl From<AppType> for SubType {
    fn from(ty: AppType) -> Self {
        SubType::App(ty)
    }
}

impl From<DataType> for SubType {
    fn from(ty: DataType) -> Self {
        SubType::Data(ty)
    }
}

impl From<u8> for SubType {
    fn from(ty: u8) -> Self {
        SubType::Custom(ty)
    }
}

impl SubType {
    /// Return the numeric partition type ID for the given subtype
    pub fn as_u8(&self) -> u8 {
        match self {
            SubType::App(ty) => *ty as u8,
            SubType::Data(ty) => *ty as u8,
            SubType::Custom(ty) => *ty,
        }
    }
}

/// Partition sub-types which can be used with [`Type::App`] partitions
///
/// A full list of support subtypes can be found in the ESP-IDF documentation:  
/// <https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/partition-tables.html#subtype>
#[allow(non_camel_case_types)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    DekuRead,
    Deserialize,
    EnumString,
    EnumVariantNames,
    FromRepr,
    Serialize,
)]
#[deku(endian = "little", type = "u8")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AppType {
    Factory = 0x00,
    Ota_0   = 0x10,
    Ota_1   = 0x11,
    Ota_2   = 0x12,
    Ota_3   = 0x13,
    Ota_4   = 0x14,
    Ota_5   = 0x15,
    Ota_6   = 0x16,
    Ota_7   = 0x17,
    Ota_8   = 0x18,
    Ota_9   = 0x19,
    Ota_10  = 0x1A,
    Ota_11  = 0x1B,
    Ota_12  = 0x1C,
    Ota_13  = 0x1D,
    Ota_14  = 0x1E,
    Ota_15  = 0x1F,
    Test    = 0x20,
}

/// Partition sub-types which can be used with [`Type::Data`] partitions
///
/// A full list of support subtypes can be found in the ESP-IDF documentation:  
/// <https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/partition-tables.html#subtype>
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    DekuRead,
    Deserialize,
    EnumIter,
    EnumString,
    EnumVariantNames,
    FromRepr,
    Serialize,
)]
#[deku(endian = "little", type = "u8")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum DataType {
    Ota       = 0x00,
    Phy       = 0x01,
    Nvs       = 0x02,
    Coredump  = 0x03,
    NvsKeys   = 0x04,
    EfuseEm   = 0x05,
    Undefined = 0x06,
    Esphttpd  = 0x80,
    Fat       = 0x81,
    Spiffs    = 0x82,
}

#[derive(Debug, DekuRead)]
#[deku(endian = "little", magic = b"\xAA\x50")]
pub(crate) struct DeserializedBinPartition {
    ty: u8,
    subtype: u8,
    offset: Option<u32>,
    size: u32,
    name: [u8; MAX_NAME_LEN + 1], // Extra byte for the NULL terminator!
    encrypted: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DeserializedCsvPartition {
    #[serde(deserialize_with = "deserialize_partition_name")]
    name: String,
    #[serde(deserialize_with = "deserialize_partition_type")]
    ty: Type,
    #[serde(deserialize_with = "deserialize_partition_subtype")]
    subtype: SubType,
    #[serde(deserialize_with = "deserialize_partition_offset")]
    offset: Option<u32>,
    #[serde(deserialize_with = "deserialize_partition_size")]
    size: u32,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_partition_flags")]
    encrypted: bool,
}

impl DeserializedCsvPartition {
    /// Ensure that the `offset` field is set (and is correctly aligned)
    pub(crate) fn fix_offset(&mut self, offset: u32) -> u32 {
        if self.offset.is_none() {
            let alignment = if self.ty == Type::App {
                PARTITION_ALIGNMENT
            } else {
                4 // 4 bytes, 32 bits
            };

            let offset = if offset % alignment != 0 {
                offset + alignment - (offset % alignment)
            } else {
                offset
            };

            self.offset = Some(offset);
        }

        self.offset.unwrap() + self.size
    }
}

/// A partition
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Partition {
    name: String,
    ty: Type,
    subtype: SubType,
    offset: u32,
    size: u32,
    encrypted: bool,
}

impl Partition {
    /// Construct a new partition
    pub fn new(
        name: String,
        ty: Type,
        subtype: SubType,
        offset: u32,
        size: u32,
        encrypted: bool,
    ) -> Self {
        Self {
            name,
            ty,
            subtype,
            offset,
            size,
            encrypted,
        }
    }

    /// Return the partition's name
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Return the partition's [Type]
    pub fn ty(&self) -> Type {
        self.ty
    }

    /// Return the partition's [SubType]
    pub fn subtype(&self) -> SubType {
        self.subtype
    }

    /// Return the partition's offset
    pub fn offset(&self) -> u32 {
        self.offset
    }

    /// Return the partition's size
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Is the partition encrypted?
    pub fn encrypted(&self) -> bool {
        self.encrypted
    }

    /// Does this partition overlap with another?
    pub fn overlaps(&self, other: &Partition) -> bool {
        max(self.offset, other.offset) < min(self.offset + self.size, other.offset + other.size)
    }

    /// Write a record to the provided binary writer
    pub fn write_bin<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: Write,
    {
        writer.write_all(&MAGIC_BYTES)?;
        writer.write_all(&[self.ty.as_u8(), self.subtype.as_u8()])?;
        writer.write_all(&self.offset.to_le_bytes())?;
        writer.write_all(&self.size.to_le_bytes())?;

        let mut name_bytes = [0u8; 16];
        for (source, dest) in self.name.bytes().zip(name_bytes.iter_mut()) {
            *dest = source;
        }
        writer.write_all(&name_bytes)?;

        writer.write_all(&(self.encrypted as u32).to_le_bytes())?;

        Ok(())
    }

    /// Write a record to the provided [`csv::Writer`]
    pub fn write_csv<W>(&self, csv: &mut csv::Writer<W>) -> std::io::Result<()>
    where
        W: Write,
    {
        let flags = if self.encrypted { "encrypted" } else { "" };

        csv.write_record(&[
            self.name(),
            self.ty.to_string(),
            self.subtype.to_string(),
            format!("{:#x}", self.offset),
            format!("{:#x}", self.size),
            flags.to_string(),
        ])?;

        Ok(())
    }
}

impl From<DeserializedBinPartition> for Partition {
    fn from(part: DeserializedBinPartition) -> Self {
        assert!(part.offset.is_some());

        let ty = Type::from(part.ty);
        let subtype = match ty {
            Type::App => SubType::from(AppType::from_repr(part.subtype.into()).unwrap()),
            Type::Data => SubType::from(DataType::from_repr(part.subtype.into()).unwrap()),
            Type::Custom(..) => SubType::from(part.subtype),
        };

        Self {
            name: String::from_utf8_lossy(&part.name)
                .to_string()
                .trim_matches(char::from(0))
                .to_string(),
            ty,
            subtype,
            offset: part.offset.unwrap(),
            size: part.size,
            encrypted: part.encrypted,
        }
    }
}

impl From<DeserializedCsvPartition> for Partition {
    fn from(part: DeserializedCsvPartition) -> Self {
        assert!(part.offset.is_some());

        Self {
            name: part.name.trim_matches(char::from(0)).to_string(),
            ty: part.ty,
            subtype: part.subtype,
            offset: part.offset.unwrap(),
            size: part.size,
            encrypted: part.encrypted,
        }
    }
}

fn deserialize_partition_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    let bytes = if let Some((idx, _)) = buf.as_str().char_indices().nth(MAX_NAME_LEN) {
        buf[..idx].as_bytes()
    } else {
        buf.as_bytes()
    };

    let mut name_bytes = [0u8; MAX_NAME_LEN + 1]; // Extra byte for the NULL terminator!
    for (source, dest) in bytes.iter().zip(name_bytes.iter_mut()) {
        *dest = *source;
    }

    Ok(String::from_utf8_lossy(&name_bytes).to_string())
}

fn deserialize_partition_type<'de, D>(deserializer: D) -> Result<Type, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    let buf = buf.as_str();

    let maybe_parsed = parse_int::parse::<u8>(buf);

    if buf == "app" || maybe_parsed == Ok(0x00) {
        Ok(Type::App)
    } else if buf == "data" || maybe_parsed == Ok(0x01) {
        Ok(Type::Data)
    } else if let Ok(ty) = maybe_parsed {
        Ok(Type::Custom(ty))
    } else {
        Err(Error::custom("invalid partition type"))
    }
}

fn deserialize_partition_subtype<'de, D>(deserializer: D) -> Result<SubType, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    if let Ok(ty) = AppType::from_str(&buf) {
        Ok(SubType::App(ty))
    } else if let Ok(ty) = DataType::from_str(&buf) {
        Ok(SubType::Data(ty))
    } else if let Ok(ty) = parse_int::parse::<u8>(&buf) {
        Ok(SubType::Custom(ty))
    } else {
        Err(Error::custom("invalid partition subtype"))
    }
}

fn deserialize_partition_offset<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_partition_offset_or_size(deserializer)
}

fn deserialize_partition_size<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_partition_offset_or_size(deserializer)?
        .ok_or_else(|| Error::custom("invalid partition size/offset format"))
}

fn deserialize_partition_flags<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    Ok(buf.trim_matches(char::from(0)) == "encrypted")
}

fn deserialize_partition_offset_or_size<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    // Begins with one or more digits, optionally followed by a single 'k' or 'm'
    // character (case-insenstive, exclusive or)
    let re = Regex::new(r"(?i)^(\d+)([km]{1})$").unwrap();

    let buf = String::deserialize(deserializer)?;
    if buf.is_empty() {
        Ok(None)
    } else if let Ok(integer) = parse_int::parse::<u32>(&buf) {
        Ok(Some(integer))
    } else if let Some(captures) = re.captures(&buf) {
        // Size multiplier format (1k, 2M, etc.)
        let digits = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
        let multiplier = match captures.get(2).unwrap().as_str() {
            "k" | "K" => 1024,
            "m" | "M" => 1024 * 1024,
            _ => unreachable!(),
        };

        Ok(Some(digits * multiplier))
    } else {
        Err(Error::custom("invalid partition size/offset format"))
    }
}

#[cfg(test)]
mod tests {
    use serde::de::{
        value::{Error as ValueError, StrDeserializer},
        IntoDeserializer,
    };

    use super::*;

    #[test]
    fn test_deserialize_partition_name() {
        let deserializer: StrDeserializer<ValueError> = "factory".into_deserializer();
        assert_eq!(
            deserialize_partition_name(deserializer),
            Ok(String::from("factory\0\0\0\0\0\0\0\0\0\0"))
        );

        // Make sure long names are truncated!
        let deserializer: StrDeserializer<ValueError> =
            "areallylongpartitionname".into_deserializer();
        let result = deserialize_partition_name(deserializer);
        assert_eq!(result, Ok(String::from("areallylongparti\0")));
        assert_eq!(result.unwrap().len(), 17);
    }

    #[test]
    fn test_deserialize_partition_type() {
        let deserializer: StrDeserializer<ValueError> = "app".into_deserializer();
        assert_eq!(deserialize_partition_type(deserializer), Ok(Type::App));

        let deserializer: StrDeserializer<ValueError> = "data".into_deserializer();
        assert_eq!(deserialize_partition_type(deserializer), Ok(Type::Data));

        let deserializer: StrDeserializer<ValueError> = "0x40".into_deserializer();
        assert_eq!(
            deserialize_partition_type(deserializer),
            Ok(Type::Custom(0x40))
        );

        // Make sure 0x00 and 0x01 map to Type::App and Type::Data respectively if
        // provided
        let deserializer: StrDeserializer<ValueError> = "0x00".into_deserializer();
        assert_eq!(deserialize_partition_type(deserializer), Ok(Type::App));
        let deserializer: StrDeserializer<ValueError> = "0x01".into_deserializer();
        assert_eq!(deserialize_partition_type(deserializer), Ok(Type::Data));
    }

    #[test]
    fn test_deserialize_partition_subtype() {
        let deserializer: StrDeserializer<ValueError> = "factory".into_deserializer();
        assert_eq!(
            deserialize_partition_subtype(deserializer),
            Ok(SubType::App(AppType::Factory))
        );

        let deserializer: StrDeserializer<ValueError> = "nvs".into_deserializer();
        assert_eq!(
            deserialize_partition_subtype(deserializer),
            Ok(SubType::Data(DataType::Nvs))
        );

        let deserializer: StrDeserializer<ValueError> = "0x40".into_deserializer();
        assert_eq!(
            deserialize_partition_subtype(deserializer),
            Ok(SubType::Custom(0x40))
        );
    }

    #[test]
    fn test_deserialize_partition_flags() {
        let deserializer: StrDeserializer<ValueError> = "encrypted".into_deserializer();
        assert_eq!(deserialize_partition_flags(deserializer), Ok(true));

        let deserializer: StrDeserializer<ValueError> = "".into_deserializer();
        assert_eq!(deserialize_partition_flags(deserializer), Ok(false));

        let deserializer: StrDeserializer<ValueError> = "foo".into_deserializer();
        assert_eq!(deserialize_partition_flags(deserializer), Ok(false));
    }

    #[test]
    fn test_deserialize_partition_offset_or_size() {
        let deserializer: StrDeserializer<ValueError> = "16384".into_deserializer();
        assert_eq!(
            deserialize_partition_offset_or_size(deserializer),
            Ok(Some(16384))
        );

        let deserializer: StrDeserializer<ValueError> = "0x9000".into_deserializer();
        assert_eq!(
            deserialize_partition_offset_or_size(deserializer),
            Ok(Some(0x9000))
        );

        let deserializer: StrDeserializer<ValueError> = "4k".into_deserializer();
        assert_eq!(
            deserialize_partition_offset_or_size(deserializer),
            Ok(Some(4096))
        );

        let deserializer: StrDeserializer<ValueError> = "1M".into_deserializer();
        assert_eq!(
            deserialize_partition_offset_or_size(deserializer),
            Ok(Some(1024 * 1024))
        );

        // Offsets can optionally be omitted in some cases
        let deserializer: StrDeserializer<ValueError> = "".into_deserializer();
        assert_eq!(deserialize_partition_offset_or_size(deserializer), Ok(None));
    }
}
