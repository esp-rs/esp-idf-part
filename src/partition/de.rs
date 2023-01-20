use std::str::FromStr;

use deku::DekuRead;
use regex::Regex;
use serde::{de::Error, Deserialize, Deserializer};

use super::{AppType, DataType, Partition, SubType, Type, APP_PARTITION_ALIGNMENT, MAX_NAME_LEN};

#[derive(Debug, DekuRead)]
#[deku(endian = "little", magic = b"\xAA\x50")]
pub(crate) struct DeserializedBinPartition {
    ty: u8,
    subtype: u8,
    offset: Option<u32>,
    size: u32,
    name: [u8; MAX_NAME_LEN],
    encrypted: bool,
}

impl From<DeserializedBinPartition> for Partition {
    fn from(part: DeserializedBinPartition) -> Self {
        assert!(part.offset.is_some());

        let ty = Type::from(part.ty);
        let subtype = match ty {
            Type::App => SubType::app(part.subtype),
            Type::Data => SubType::data(part.subtype),
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

impl From<DeserializedCsvPartition> for Partition {
    fn from(part: DeserializedCsvPartition) -> Self {
        assert!(part.offset.is_some());

        let subtype = match part.ty {
            Type::App => SubType::app(part.subtype.into()),
            Type::Data => SubType::data(part.subtype.into()),
            Type::Custom(..) => part.subtype,
        };

        Self {
            name: part.name.trim_matches(char::from(0)).to_string(),
            ty: part.ty,
            subtype,
            offset: part.offset.unwrap(),
            size: part.size,
            encrypted: part.encrypted,
        }
    }
}

impl DeserializedCsvPartition {
    /// Ensure that the `offset` field is set (and is correctly aligned)
    pub(crate) fn fix_offset(&mut self, offset: u32) -> u32 {
        if self.offset.is_none() {
            let alignment = if self.ty == Type::App {
                APP_PARTITION_ALIGNMENT
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
