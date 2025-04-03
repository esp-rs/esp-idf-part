use core::cmp::{max, min};

use deku::DekuRead;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumString, FromRepr, IntoEnumIterator, VariantNames};

pub(crate) use self::de::{DeserializedBinPartition, DeserializedCsvPartition};

mod de;

pub(crate) const APP_PARTITION_ALIGNMENT: u32 = 0x10000;
pub(crate) const DATA_PARTITION_ALIGNMENT: u32 = 0x1000;
pub(crate) const MAX_NAME_LEN: usize = 16;

/// Supported partition types
///
/// User-defined partition types are allowed as long as their type ID does not
/// confict with [`Type::App`] or [`Type::Data`]. Custom type IDs must not
/// exceed `0xFE`.
///
/// For additional information regarding the supported partition types, please
/// refer to the ESP-IDF documentation:
/// <https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/partition-tables.html#type-field>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, DekuRead)]
#[deku(endian = "little", id_type = "u8")]
#[serde(rename_all = "lowercase")]
pub enum Type {
    #[deku(id = "0x00")]
    App,
    #[deku(id = "0x01")]
    Data,
    #[deku(id_pat = "0x02..=0xFE")]
    Custom(u8),
}

impl core::fmt::Display for Type {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Type::App | Type::Data => serde_plain::to_string(self).unwrap(),
                Type::Custom(ty) => serde_plain::to_string(&format_args!("{:#04x}", ty)).unwrap(),
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

impl From<Type> for u8 {
    fn from(value: Type) -> Self {
        match value {
            Type::App => 0x00,
            Type::Data => 0x01,
            Type::Custom(ty) => ty,
        }
    }
}

impl Type {
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

impl core::fmt::Display for SubType {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SubType::App(ty) => serde_plain::to_string(ty).unwrap(),
                SubType::Data(ty) => serde_plain::to_string(ty).unwrap(),
                SubType::Custom(ty) =>
                    serde_plain::to_string(&format_args!("{:#04x}", ty)).unwrap(),
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

impl From<SubType> for u8 {
    fn from(value: SubType) -> Self {
        match value {
            SubType::App(ty) => ty as u8,
            SubType::Data(ty) => ty as u8,
            SubType::Custom(ty) => ty,
        }
    }
}

impl SubType {
    /// Create a [SubType::App] variant from an integer value
    pub fn app(value: u8) -> Self {
        Self::App(AppType::from_repr(value as usize).unwrap())
    }

    /// Create a [SubType::Data] variant from an integer value
    pub fn data(value: u8) -> Self {
        Self::Data(DataType::from_repr(value as usize).unwrap())
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
    Deserialize,
    EnumIter,
    EnumString,
    VariantNames,
    FromRepr,
    Serialize,
    DekuRead,
)]
#[deku(endian = "little", id_type = "u8")]
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
    Deserialize,
    EnumIter,
    EnumString,
    VariantNames,
    FromRepr,
    Serialize,
    DekuRead,
)]
#[deku(endian = "little", id_type = "u8")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum DataType {
    Ota       = 0x00,
    Phy       = 0x01,
    Nvs       = 0x02,
    Coredump  = 0x03,
    NvsKeys   = 0x04,
    #[serde(rename = "efuse")]
    #[strum(serialize = "efuse")]
    EfuseEm   = 0x05,
    Undefined = 0x06,
    Esphttpd  = 0x80,
    Fat       = 0x81,
    Spiffs    = 0x82,
    Littlefs  = 0x83,
}

bitflags::bitflags! {
    /// Supported partition flags
    ///
    /// Two flags are currently supported, `encrypted` and `readonly`:
    ///
    /// - If `encrypted` flag is set, the partition will be encrypted if [Flash
    ///   Encryption] is enabled.
    ///     - Note: `app` type partitions will always be encrypted, regardless of
    ///       whether this flag is set or not.
    /// - If `readonly` flag is set, the partition will be read-only. This flag is
    ///   only supported for `data` type partitions except `ota` and `coredump`
    ///   subtypes. This flag can help to protect against accidental writes to a
    ///   partition that contains critical device-specific configuration data, e.g.
    ///   factory data partition.
    ///
    /// You can specify multiple flags by separating them with a colon. For example,
    /// `encrypted:readonly`.
    ///
    /// For more information, see the ESP-IDF documentation:
    /// <https://docs.espressif.com/projects/esp-idf/en/v5.3.1/esp32/api-guides/partition-tables.html#flags>
    ///
    /// [Flash Encryption]: https://docs.espressif.com/projects/esp-idf/en/v5.3.1/esp32/security/flash-encryption.html
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
    pub struct Flags: u32 {
        /// Encrypted partition
        const ENCRYPTED = 0b0001;
        /// Read-only partition
        const READONLY  = 0b0010;
    }
}

/// A single partition definition
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Partition {
    name: String,
    ty: Type,
    subtype: SubType,
    offset: u32,
    size: u32,
    flags: Flags,
}

impl Partition {
    /// Construct a new partition
    pub fn new<S>(name: S, ty: Type, subtype: SubType, offset: u32, size: u32, flags: Flags) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            ty,
            subtype,
            offset,
            size,
            flags,
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

    /// Return the partition's flags
    pub fn flags(&self) -> Flags {
        self.flags
    }

    /// Does this partition overlap with another?
    pub fn overlaps(&self, other: &Partition) -> bool {
        max(self.offset, other.offset) < min(self.offset + self.size, other.offset + other.size)
    }

    /// Write a record to the provided binary writer
    pub fn write_bin<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        const MAGIC_BYTES: [u8; 2] = [0xAA, 0x50];

        writer.write_all(&MAGIC_BYTES)?;
        writer.write_all(&[self.ty.into(), self.subtype.into()])?;
        writer.write_all(&self.offset.to_le_bytes())?;
        writer.write_all(&self.size.to_le_bytes())?;

        let mut name_bytes = [0u8; 16];
        for (source, dest) in self.name.bytes().zip(name_bytes.iter_mut()) {
            *dest = source;
        }
        writer.write_all(&name_bytes)?;

        writer.write_all(&self.flags.bits().to_le_bytes())?;

        Ok(())
    }

    /// Write a record to the provided [`csv::Writer`]
    pub fn write_csv<W>(&self, csv: &mut csv::Writer<W>) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        let mut flags = Vec::<&str>::new();
        if self.flags.contains(Flags::ENCRYPTED) {
            flags.push("encrypted");
        }
        if self.flags.contains(Flags::READONLY) {
            flags.push("readonly");
        }

        let flags = flags.join(":");

        csv.write_record(&[
            self.name(),
            self.ty.to_string(),
            self.subtype.to_string(),
            format!("{:#x}", self.offset),
            format!("{:#x}", self.size),
            flags,
        ])?;

        Ok(())
    }
}
