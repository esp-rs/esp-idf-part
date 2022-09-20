use core::fmt::Display;

use regex::Regex;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

/// Partition type
/// <https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/partition-tables.html#type-field>
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    App,
    Data,
    Custom(u8),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Type::App | Type::Data => serde_plain::to_string(self).unwrap(),
                Type::Custom(ty) => format!("{:#04x}", ty),
            }
        )
    }
}

/// Partition sub-type
/// <https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/partition-tables.html#subtype>
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SubType {
    App(AppType),
    Data(DataType),
    #[serde(deserialize_with = "deserialize_custom_partition_sub_type")]
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

/// Partition sub-types which can be used with App partitions
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
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

/// Partition sub-types which can be used with Data partitions
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
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

/// Partition flags
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Flags {
    Encrypted = 0x01,
}

/// A partition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Partition {
    #[serde(deserialize_with = "deserialize_partition_name")]
    name: String,
    #[serde(deserialize_with = "deserialize_partition_type")]
    ty: Type,
    subtype: SubType,
    #[serde(deserialize_with = "deserialize_partition_offset")]
    offset: Option<u32>,
    #[serde(deserialize_with = "deserialize_partition_size")]
    size: u32,
    flags: Option<Flags>,
}

impl Partition {
    pub fn new(
        name: String,
        ty: Type,
        subtype: SubType,
        offset: Option<u32>,
        size: u32,
        flags: Option<Flags>,
    ) -> Self {
        Self {
            name,
            ty,
            subtype,
            offset,
            size,
            flags,
        }
    }
}

fn deserialize_partition_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    // Partition names longer than 16 bytes are truncated; this limitation
    // *includes* the terminating NULL byte, so we will truncate to 15 bytes
    // instead.
    //
    // <https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/partition-tables.html#name-field>
    const MAX_LENGTH: usize = 15;

    let buf = String::deserialize(deserializer)?;

    if let Some((idx, _)) = buf.as_str().char_indices().nth(MAX_LENGTH) {
        Ok(String::from(&buf[..idx]))
    } else {
        Ok(buf)
    }
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
    } else if let Ok(integer) = maybe_parsed {
        Ok(Type::Custom(integer))
    } else {
        Err(Error::custom("invalid partition type"))
    }
}

fn deserialize_custom_partition_sub_type<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    parse_int::parse::<u8>(&buf).map_err(|_| Error::custom("invalid data partition sub-type"))
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
            Ok(String::from("factory"))
        );

        // Make sure long names are truncated!
        let deserializer: StrDeserializer<ValueError> =
            "areallylongpartitionname".into_deserializer();
        let result = deserialize_partition_name(deserializer);
        assert_eq!(result, Ok(String::from("areallylongpart")));
        assert_eq!(result.unwrap().len(), 15);
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
    fn test_deserialize_custom_partition_sub_type() {
        let deserializer: StrDeserializer<ValueError> = "0x00".into_deserializer();
        assert_eq!(
            deserialize_custom_partition_sub_type(deserializer),
            Ok(0x00)
        );
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
