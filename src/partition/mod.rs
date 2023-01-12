use core::cmp::{max, min};

#[cfg(feature = "std")]
use deku::{DekuEnumExt, DekuError, DekuRead};
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use strum::IntoEnumIterator;
use strum::{EnumIter, EnumString, EnumVariantNames, FromRepr};

#[cfg(feature = "std")]
pub(crate) use self::de::{DeserializedBinPartition, DeserializedCsvPartition};

#[cfg(feature = "std")]
mod de;

#[cfg(not(feature = "std"))]
type String = heapless::String<MAX_NAME_LEN>;

#[cfg(feature = "std")]
const MAGIC_BYTES: [u8; 2] = [0xAA, 0x50];
pub(crate) const MAX_NAME_LEN: usize = 16;
#[cfg(feature = "std")]
pub(crate) const PARTITION_ALIGNMENT: u32 = 0x10000;

/// Supported partition types
///
/// User-defined partition types are allowed as long as their type ID does not
/// confict with [`Type::App`] or [`Type::Data`].
///
/// For additional information regarding the supported partition types, please
/// refer to the ESP-IDF documentation:  
/// <https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/partition-tables.html#type-field>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(
    feature = "std",
    derive(DekuRead),
    deku(endian = "little", type = "u8")
)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    #[cfg_attr(feature = "std", deku(id = "0x00"))]
    App,
    #[cfg_attr(feature = "std", deku(id = "0x01"))]
    Data,
    #[cfg_attr(feature = "std", deku(id_pat = "0x02..=0xFE"))]
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

impl Type {
    /// Return the numeric partition type ID for the given type
    pub fn as_u8(&self) -> u8 {
        match self {
            Type::App => 0x00,
            Type::Data => 0x01,
            Type::Custom(ty) => *ty,
        }
    }

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
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
    Deserialize,
    EnumIter,
    EnumString,
    EnumVariantNames,
    FromRepr,
    Serialize,
)]
#[cfg_attr(
    feature = "std",
    derive(DekuRead),
    deku(endian = "little", type = "u8")
)]
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
    EnumVariantNames,
    FromRepr,
    Serialize,
)]
#[cfg_attr(
    feature = "std",
    derive(DekuRead),
    deku(endian = "little", type = "u8")
)]
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

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    /// Write a record to the provided binary writer
    pub fn write_bin<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
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

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    /// Write a record to the provided [`csv::Writer`]
    pub fn write_csv<W>(&self, csv: &mut csv::Writer<W>) -> std::io::Result<()>
    where
        W: std::io::Write,
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
