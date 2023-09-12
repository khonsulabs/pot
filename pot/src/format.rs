use std::fmt::Display;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use half::f16;

pub(crate) const CURRENT_VERSION: u8 = 0;

use crate::reader::{BufferedBytes, Reader};
use crate::Error;
/// Writes an atom header into `writer`.
#[allow(clippy::cast_possible_truncation)]
#[inline]
fn write_tiny_atom_header<W: WriteBytesExt>(
    mut writer: W,
    kind: Kind,
    arg: u8,
) -> std::io::Result<usize> {
    // Kind is the 3 bits.
    let mut first_byte = (kind as u8) << 5;
    if arg > 0 {
        debug_assert!(arg < 0x10);
        first_byte |= arg & 0b1111;
    }

    writer.write_all(&[first_byte])?;
    Ok(1)
}

/// Writes an atom header into `writer`.
#[allow(clippy::cast_possible_truncation)]
#[inline]
pub fn write_atom_header<W: WriteBytesExt>(
    mut writer: W,
    kind: Kind,
    mut arg: u64,
) -> std::io::Result<usize> {
    if arg < 0x10 {
        write_tiny_atom_header(writer, kind, arg as u8)
    } else {
        // Kind is the 3 bits.
        let mut first_byte = (kind as u8) << 5;
        // The last 4 bits are the first 4 bits of the arg. We also know
        // that we're longer than one byte, due to the original match.
        first_byte |= arg as u8 & 0b1111;
        arg >>= 4;
        first_byte |= 0b10000;

        let mut second = arg as u8 & 0x7F;
        arg >>= 7;
        if arg == 0 {
            writer.write_all(&[first_byte, second])?;
            return Ok(2);
        }

        second |= 0b1000_0000;
        let mut third = arg as u8 & 0x7F;
        arg >>= 7;
        if arg == 0 {
            writer.write_all(&[first_byte, second, third])?;
            return Ok(3);
        }

        third |= 0b1000_0000;
        let mut fourth = arg as u8 & 0x7F;
        arg >>= 7;
        if arg == 0 {
            writer.write_all(&[first_byte, second, third, fourth])?;
            return Ok(4);
        }

        fourth |= 0b1000_0000;
        let mut fifth = arg as u8 & 0x7F;
        arg >>= 7;
        if arg == 0 {
            writer.write_all(&[first_byte, second, third, fourth, fifth])?;
            return Ok(5);
        }

        fifth |= 0b1000_0000;
        let mut sixth = arg as u8 & 0x7F;
        arg >>= 7;
        if arg == 0 {
            writer.write_all(&[first_byte, second, third, fourth, fifth, sixth])?;
            return Ok(6);
        }
        sixth |= 0b1000_0000;
        let mut seventh = arg as u8 & 0x7F;
        arg >>= 7;
        if arg == 0 {
            writer.write_all(&[first_byte, second, third, fourth, fifth, sixth, seventh])?;
            return Ok(7);
        }
        seventh |= 0b1000_0000;
        let mut eighth = arg as u8 & 0x7F;
        arg >>= 7;
        if arg == 0 {
            writer.write_all(&[
                first_byte, second, third, fourth, fifth, sixth, seventh, eighth,
            ])?;
            return Ok(8);
        }

        eighth |= 0b1000_0000;
        let mut ninth = arg as u8 & 0x7F;
        arg >>= 7;
        if arg == 0 {
            writer.write_all(&[
                first_byte, second, third, fourth, fifth, sixth, seventh, eighth, ninth,
            ])?;
            return Ok(9);
        }

        ninth |= 0b1000_0000;
        debug_assert!(arg <= 255);
        writer.write_all(&[
            first_byte, second, third, fourth, fifth, sixth, seventh, eighth, ninth, arg as u8,
        ])?;
        Ok(10)
    }
}

/// Reads an atom header (kind and argument).
#[inline]
pub fn read_atom_header<R: ReadBytesExt>(reader: &mut R) -> Result<(Kind, u64), Error> {
    let first_byte = reader.read_u8()?;
    let kind = Kind::from_u8(first_byte >> 5)?;
    let mut arg = u64::from(first_byte & 0b1111);
    if first_byte & 0b10000 != 0 {
        let mut bytes_remaining = 9;
        let mut offset = 4;
        loop {
            let byte = reader.read_u8()?;
            let data = byte & 0x7f;
            arg |= u64::from(data) << offset;
            offset += 7;
            bytes_remaining -= 1;
            if data == byte || bytes_remaining == 0 {
                break;
            }
        }
    }

    Ok((kind, arg))
}

/// The type of an atom.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Kind {
    /// A value with a special meaning.
    Special = 0,
    /// A signed integer. Argument is the byte length, minus one. The following
    /// bytes are the value, stored in little endian.
    Int = 1,
    /// An unsigned integer. Argument is the byte length, minus one. The
    /// following bytes are the value, stored in little endian.
    UInt = 2,
    /// A floating point value. Argument is the byte length, minus one. Must be
    /// either 2, 4 or 8 bytes. The following bytes are the value, stored in
    /// little endian. The two-byte representation follows the IEEE 754-2008
    /// standard, implemented by the [`half`] crate.
    Float = 3,
    /// A list of atoms. Argument is the count of atoms in the sequence.
    Sequence = 4,
    /// A list of key-value pairs. Argument is the count of entries in the map.
    /// There will be twice as many total atoms, since each entry is a key/value
    /// pair.
    Map = 5,
    /// A symbol. If the least-significant bit of the arg is 0, this is a new
    /// symbol. The remaining bits of the arg contain the length in bytes. The
    /// following bytes will contain the symbol bytes (UTF-8). It should be
    /// stored and given a unique symbol id, starting at 0.
    ///
    /// If the least-significant bit of the arg is 1, the remaining bits are the
    /// symbol id of a previously emitted symbol.
    Symbol = 6,
    /// A series of bytes. The argument is the length. The bytes follow.
    Bytes = 7,
}

impl Kind {
    /// Converts from a u8. Returns an error if `kind` is an invalid value.
    #[inline]
    pub const fn from_u8(kind: u8) -> Result<Self, Error> {
        match kind {
            0 => Ok(Self::Special),
            1 => Ok(Self::Int),
            2 => Ok(Self::UInt),
            3 => Ok(Self::Float),
            4 => Ok(Self::Sequence),
            5 => Ok(Self::Map),
            6 => Ok(Self::Symbol),
            7 => Ok(Self::Bytes),
            other => Err(Error::InvalidKind(other)),
        }
    }
}

/// A special value type.
#[derive(Debug)]
pub enum Special {
    /// A None value.
    None = 0,
    /// A Unit value.
    Unit = 1,
    /// The `false` boolean literal.
    False = 2,
    /// The `true` boolean literal.
    True = 3,
    /// A named value. A symbol followed by another value.
    Named = 4,
    /// A sequence of key-value pairs with an unknown length.
    DynamicMap = 5,
    /// A terminal value for a [`Self::DynamicMap`].
    DynamicEnd = 6,
}

#[cfg(test)]
pub(crate) const SPECIAL_COUNT: u64 = Special::Named as u64 + 1;

impl TryFrom<u64> for Special {
    type Error = UnknownSpecial;

    #[inline]
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Unit),
            2 => Ok(Self::False),
            3 => Ok(Self::True),
            4 => Ok(Self::Named),
            5 => Ok(Self::DynamicMap),
            6 => Ok(Self::DynamicEnd),
            _ => Err(UnknownSpecial(value)),
        }
    }
}

#[test]
fn unknown_special() {
    let err = Special::try_from(u64::MAX).unwrap_err();
    assert_eq!(err, UnknownSpecial(u64::MAX));
    assert!(err.to_string().contains("unknown special"));
}

/// An unknown [`Special`] was encountered.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct UnknownSpecial(pub u64);

impl Display for UnknownSpecial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown special id: {}", self.0)
    }
}

/// Writes the Pot header. A u32 written in big endian. The first three bytes
/// are 'Pot' (`0x506F74`), and the fourth byte is the version. The first
/// version of Pot is 0.
#[inline]
pub fn write_header<W: WriteBytesExt>(mut writer: W, version: u8) -> std::io::Result<usize> {
    writer.write_u32::<BigEndian>(0x506F_7400 | u32::from(version))?;
    Ok(4)
}

/// Reads a Pot header. See [`write_header`] for more information. Returns the version number contained within.
#[allow(clippy::similar_names, clippy::cast_possible_truncation)]
#[inline]
pub fn read_header<R: ReadBytesExt>(reader: &mut R) -> Result<u8, Error> {
    let header = reader.read_u32::<BigEndian>()?;
    if header & 0x506F_7400 == 0x506F_7400 {
        let version = (header & 0xFF) as u8;
        Ok(version)
    } else {
        Err(Error::IncompatibleVersion)
    }
}
/// Writes a [`Kind::Special`] atom.
#[inline]
pub fn write_special<W: WriteBytesExt>(writer: W, special: Special) -> std::io::Result<usize> {
    write_atom_header(writer, Kind::Special, special as u64)
}

/// Writes a [`Kind::Special`] atom with [`Special::None`].
#[inline]
pub fn write_none<W: WriteBytesExt>(writer: W) -> std::io::Result<usize> {
    write_special(writer, Special::None)
}

/// Writes a [`Kind::Special`] atom with [`Special::Unit`].
#[inline]
pub fn write_unit<W: WriteBytesExt>(writer: W) -> std::io::Result<usize> {
    write_special(writer, Special::Unit)
}

/// Writes a [`Kind::Special`] atom with [`Special::Named`].
#[inline]
pub fn write_named<W: WriteBytesExt>(writer: W) -> std::io::Result<usize> {
    write_special(writer, Special::Named)
}

/// Writes a [`Kind::Special`] atom with either [`Special::True`] or [`Special::False`].
#[inline]
pub fn write_bool<W: WriteBytesExt>(writer: W, boolean: bool) -> std::io::Result<usize> {
    write_special(
        writer,
        if boolean {
            Special::True
        } else {
            Special::False
        },
    )
}

/// Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible.
#[inline]
pub fn write_i8<W: WriteBytesExt>(mut writer: W, value: i8) -> std::io::Result<usize> {
    let header_len =
        write_atom_header(&mut writer, Kind::Int, std::mem::size_of::<i8>() as u64 - 1)?;
    writer
        .write_i8(value)
        .map(|_| std::mem::size_of::<i8>() + header_len)
}

/// Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible.
#[inline]
pub fn write_i16<W: WriteBytesExt>(mut writer: W, value: i16) -> std::io::Result<usize> {
    if let Ok(value) = i8::try_from(value) {
        write_i8(writer, value)
    } else {
        let header_len = write_tiny_atom_header(&mut writer, Kind::Int, 2 - 1)?;
        writer
            .write_i16::<LittleEndian>(value)
            .map(|_| 2 + header_len)
    }
}

/// Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible.
#[inline]
pub fn write_i24<W: WriteBytesExt>(mut writer: W, value: i32) -> std::io::Result<usize> {
    if let Ok(value) = i16::try_from(value) {
        write_i16(writer, value)
    } else {
        let header_len = write_tiny_atom_header(&mut writer, Kind::Int, 3 - 1)?;
        writer
            .write_i24::<LittleEndian>(value)
            .map(|_| 3 + header_len)
    }
}

/// Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible.
#[inline]
pub fn write_i32<W: WriteBytesExt>(mut writer: W, value: i32) -> std::io::Result<usize> {
    if value >= -(2_i32.pow(23)) && value < 2_i32.pow(23) {
        write_i24(writer, value)
    } else {
        let header_len = write_tiny_atom_header(&mut writer, Kind::Int, 4 - 1)?;
        writer
            .write_i32::<LittleEndian>(value)
            .map(|_| 4 + header_len)
    }
}

/// Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible.
#[inline]
pub fn write_i48<W: WriteBytesExt>(mut writer: W, value: i64) -> std::io::Result<usize> {
    if let Ok(value) = i32::try_from(value) {
        write_i32(writer, value)
    } else {
        let header_len = write_tiny_atom_header(&mut writer, Kind::Int, 6 - 1)?;
        writer
            .write_i48::<LittleEndian>(value)
            .map(|_| 6 + header_len)
    }
}

/// Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible.
#[inline]
pub fn write_i64<W: WriteBytesExt>(mut writer: W, value: i64) -> std::io::Result<usize> {
    if value >= -(2_i64.pow(47)) && value < 2_i64.pow(47) {
        write_i48(writer, value)
    } else {
        let header_len = write_tiny_atom_header(&mut writer, Kind::Int, 8 - 1)?;
        writer
            .write_i64::<LittleEndian>(value)
            .map(|_| 8 + header_len)
    }
}

/// Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible.
#[inline]
pub fn write_i128<W: WriteBytesExt>(mut writer: W, value: i128) -> std::io::Result<usize> {
    if let Ok(value) = i64::try_from(value) {
        write_i64(writer, value)
    } else {
        let header_len = write_tiny_atom_header(&mut writer, Kind::Int, 16 - 1)?;
        writer
            .write_i128::<LittleEndian>(value)
            .map(|_| 16 + header_len)
    }
}

/// Writes an [`Kind::UInt`] atom with the given value.
#[inline]
pub fn write_u8<W: WriteBytesExt>(mut writer: W, value: u8) -> std::io::Result<usize> {
    let header_len = write_tiny_atom_header(&mut writer, Kind::UInt, 0)?;
    writer
        .write_u8(value)
        .map(|_| std::mem::size_of::<u8>() + header_len)
}

/// Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible.
#[inline]
pub fn write_u16<W: WriteBytesExt>(mut writer: W, value: u16) -> std::io::Result<usize> {
    if let Ok(value) = u8::try_from(value) {
        write_u8(writer, value)
    } else {
        let header_len = write_tiny_atom_header(&mut writer, Kind::UInt, 1)?;
        writer
            .write_u16::<LittleEndian>(value)
            .map(|_| std::mem::size_of::<u16>() + header_len)
    }
}

/// Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible.
#[inline]
pub fn write_u24<W: WriteBytesExt>(mut writer: W, value: u32) -> std::io::Result<usize> {
    if let Ok(value) = u16::try_from(value) {
        write_u16(writer, value)
    } else {
        let header_len = write_tiny_atom_header(&mut writer, Kind::UInt, 2)?;
        writer
            .write_u24::<LittleEndian>(value)
            .map(|_| 3 + header_len)
    }
}

/// Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible.
#[inline]
pub fn write_u32<W: WriteBytesExt>(mut writer: W, value: u32) -> std::io::Result<usize> {
    if value < 2_u32.pow(24) {
        write_u24(writer, value)
    } else {
        let header_len = write_tiny_atom_header(&mut writer, Kind::UInt, 3)?;
        writer
            .write_u32::<LittleEndian>(value)
            .map(|_| std::mem::size_of::<u32>() + header_len)
    }
}

/// Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible.
#[inline]
pub fn write_u48<W: WriteBytesExt>(mut writer: W, value: u64) -> std::io::Result<usize> {
    if let Ok(value) = u32::try_from(value) {
        write_u32(writer, value)
    } else {
        let header_len = write_tiny_atom_header(&mut writer, Kind::UInt, 5)?;
        writer
            .write_u48::<LittleEndian>(value)
            .map(|_| 6 + header_len)
    }
}

/// Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible.
#[inline]
pub fn write_u64<W: WriteBytesExt>(mut writer: W, value: u64) -> std::io::Result<usize> {
    if value < 2_u64.pow(48) {
        write_u48(writer, value)
    } else {
        let header_len = write_tiny_atom_header(&mut writer, Kind::UInt, 7)?;
        writer
            .write_u64::<LittleEndian>(value)
            .map(|_| std::mem::size_of::<u64>() + header_len)
    }
}

/// Writes an [`Kind::Int`] atom with the given value. Will encode in a smaller format if possible.
#[inline]
pub fn write_u128<W: WriteBytesExt>(mut writer: W, value: u128) -> std::io::Result<usize> {
    if let Ok(value) = u64::try_from(value) {
        write_u64(writer, value)
    } else {
        let header_len = write_tiny_atom_header(&mut writer, Kind::UInt, 15)?;
        writer
            .write_u128::<LittleEndian>(value)
            .map(|_| std::mem::size_of::<u128>() + header_len)
    }
}

/// Writes an [`Kind::Float`] atom with the given value.
#[inline]
#[allow(clippy::cast_possible_truncation, clippy::float_cmp)]
pub fn write_f32<W: WriteBytesExt>(mut writer: W, value: f32) -> std::io::Result<usize> {
    let as_f16 = f16::from_f32(value);
    if as_f16.to_f32() == value {
        let header_len = write_tiny_atom_header(
            &mut writer,
            Kind::Float,
            std::mem::size_of::<u16>() as u8 - 1,
        )?;
        writer
            .write_u16::<LittleEndian>(as_f16.to_bits())
            .map(|_| std::mem::size_of::<u16>() + header_len)
    } else {
        let header_len = write_tiny_atom_header(
            &mut writer,
            Kind::Float,
            std::mem::size_of::<f32>() as u8 - 1,
        )?;
        writer
            .write_f32::<LittleEndian>(value)
            .map(|_| std::mem::size_of::<f32>() + header_len)
    }
}

fn read_f16<R: ReadBytesExt>(reader: &mut R) -> std::io::Result<f32> {
    let value = f16::from_bits(reader.read_u16::<LittleEndian>()?);
    Ok(value.to_f32())
}

/// Writes an [`Kind::Float`] atom with the given value.
#[allow(clippy::cast_possible_truncation, clippy::float_cmp)]
#[inline]
pub fn write_f64<W: WriteBytesExt>(mut writer: W, value: f64) -> std::io::Result<usize> {
    let as_f32 = value as f32;
    if f64::from(as_f32) == value {
        write_f32(writer, as_f32)
    } else {
        let header_len = write_tiny_atom_header(&mut writer, Kind::Float, 7)?;
        writer
            .write_f64::<LittleEndian>(value)
            .map(|_| std::mem::size_of::<f64>() + header_len)
    }
}

/// Writes an [`Kind::Bytes`] atom with the bytes of the string.
#[inline]
pub fn write_str<W: WriteBytesExt>(writer: W, value: &str) -> std::io::Result<usize> {
    write_bytes(writer, value.as_bytes())
}

/// Writes an [`Kind::Bytes`] atom with the given value.
#[inline]
pub fn write_bytes<W: WriteBytesExt>(mut writer: W, value: &[u8]) -> std::io::Result<usize> {
    let header_len = write_atom_header(&mut writer, Kind::Bytes, value.len() as u64)?;
    writer.write_all(value)?;
    Ok(value.len() + header_len)
}

/// An integer type that can safely convert between other number types using compile-time evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Integer(pub(crate) InnerInteger);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InnerInteger {
    /// An i8 value.
    I8(i8),
    /// An i16 value.
    I16(i16),
    /// An i32 value.
    I32(i32),
    /// An i64 value.
    I64(i64),
    /// An i128 value.
    I128(i128),
    /// An u8 value.
    U8(u8),
    /// An u16 value.
    U16(u16),
    /// An u32 value.
    U32(u32),
    /// An u64 value.
    U64(u64),
    /// An u128 value.
    U128(u128),
}

impl Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            InnerInteger::I8(value) => Display::fmt(value, f),
            InnerInteger::I16(value) => Display::fmt(value, f),
            InnerInteger::I32(value) => Display::fmt(value, f),
            InnerInteger::I64(value) => Display::fmt(value, f),
            InnerInteger::I128(value) => Display::fmt(value, f),
            InnerInteger::U8(value) => Display::fmt(value, f),
            InnerInteger::U16(value) => Display::fmt(value, f),
            InnerInteger::U32(value) => Display::fmt(value, f),
            InnerInteger::U64(value) => Display::fmt(value, f),
            InnerInteger::U128(value) => Display::fmt(value, f),
        }
    }
}

impl Integer {
    /// Returns true if the value contained is zero.
    #[must_use]
    #[inline]
    pub const fn is_zero(&self) -> bool {
        match &self.0 {
            InnerInteger::I8(value) => *value == 0,
            InnerInteger::I16(value) => *value == 0,
            InnerInteger::I32(value) => *value == 0,
            InnerInteger::I64(value) => *value == 0,
            InnerInteger::I128(value) => *value == 0,
            InnerInteger::U8(value) => *value == 0,
            InnerInteger::U16(value) => *value == 0,
            InnerInteger::U32(value) => *value == 0,
            InnerInteger::U64(value) => *value == 0,
            InnerInteger::U128(value) => *value == 0,
        }
    }

    /// Returns the contained value as an i8, or an error if the value is unable to fit.
    // clippy::checked_conversions: try_from isn't const, and it would demote this from a const fn.
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::checked_conversions)]
    #[inline]
    pub const fn as_i8(&self) -> Result<i8, Error> {
        match &self.0 {
            InnerInteger::I8(value) => Ok(*value),
            InnerInteger::U8(value) => {
                if *value <= i8::MAX as u8 {
                    Ok(*value as i8)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            _ => Err(Error::ImpreciseCastWouldLoseData),
        }
    }

    /// Returns the contained value as an u8, or an error if the value is unable to fit.
    #[allow(clippy::cast_sign_loss)]
    #[inline]
    pub const fn as_u8(&self) -> Result<u8, Error> {
        match &self.0 {
            InnerInteger::U8(value) => Ok(*value),
            InnerInteger::I8(value) => {
                if *value >= 0 {
                    Ok(*value as u8)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            _ => Err(Error::ImpreciseCastWouldLoseData),
        }
    }

    /// Returns the contained value as an i16, or an error if the value is unable to fit.
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::checked_conversions)]
    #[inline]
    pub const fn as_i16(&self) -> Result<i16, Error> {
        match &self.0 {
            InnerInteger::I8(value) => Ok(*value as i16),
            InnerInteger::U8(value) => Ok(*value as i16),
            InnerInteger::I16(value) => Ok(*value),
            InnerInteger::U16(value) => {
                if *value <= i16::MAX as u16 {
                    Ok(*value as i16)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            InnerInteger::U32(_)
            | InnerInteger::I32(_)
            | InnerInteger::U64(_)
            | InnerInteger::I64(_)
            | InnerInteger::U128(_)
            | InnerInteger::I128(_) => Err(Error::ImpreciseCastWouldLoseData),
        }
    }

    /// Returns the contained value as an u16, or an error if the value is unable to fit.
    #[allow(clippy::cast_sign_loss)]
    #[inline]
    pub const fn as_u16(&self) -> Result<u16, Error> {
        match &self.0 {
            InnerInteger::I8(value) => {
                if *value >= 0 {
                    Ok(*value as u16)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            InnerInteger::U8(value) => Ok(*value as u16),
            InnerInteger::U16(value) => Ok(*value),
            InnerInteger::I16(value) => {
                if *value >= 0 {
                    Ok(*value as u16)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            InnerInteger::U32(_)
            | InnerInteger::I32(_)
            | InnerInteger::U64(_)
            | InnerInteger::I64(_)
            | InnerInteger::U128(_)
            | InnerInteger::I128(_) => Err(Error::ImpreciseCastWouldLoseData),
        }
    }

    /// Returns the contained value as an i32, or an error if the value is unable to fit.
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::checked_conversions)]
    #[inline]
    pub const fn as_i32(&self) -> Result<i32, Error> {
        match &self.0 {
            InnerInteger::I8(value) => Ok(*value as i32),
            InnerInteger::U8(value) => Ok(*value as i32),
            InnerInteger::I16(value) => Ok(*value as i32),
            InnerInteger::U16(value) => Ok(*value as i32),
            InnerInteger::I32(value) => Ok(*value),
            InnerInteger::U32(value) => {
                if *value <= i32::MAX as u32 {
                    Ok(*value as i32)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            InnerInteger::U64(_)
            | InnerInteger::I64(_)
            | InnerInteger::U128(_)
            | InnerInteger::I128(_) => Err(Error::ImpreciseCastWouldLoseData),
        }
    }

    /// Returns the contained value as an u32, or an error if the value is unable to fit.
    #[allow(clippy::cast_sign_loss)]
    #[inline]
    pub const fn as_u32(&self) -> Result<u32, Error> {
        match &self.0 {
            InnerInteger::I8(value) => {
                if *value >= 0 {
                    Ok(*value as u32)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            InnerInteger::U8(value) => Ok(*value as u32),
            InnerInteger::I16(value) => {
                if *value >= 0 {
                    Ok(*value as u32)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            InnerInteger::U16(value) => Ok(*value as u32),
            InnerInteger::U32(value) => Ok(*value),
            InnerInteger::I32(value) => {
                if *value >= 0 {
                    Ok(*value as u32)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            InnerInteger::U64(_)
            | InnerInteger::I64(_)
            | InnerInteger::U128(_)
            | InnerInteger::I128(_) => Err(Error::ImpreciseCastWouldLoseData),
        }
    }

    /// Returns the contained value as an i64, or an error if the value is unable to fit.
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::checked_conversions)]
    #[inline]
    pub const fn as_i64(&self) -> Result<i64, Error> {
        match &self.0 {
            InnerInteger::I8(value) => Ok(*value as i64),
            InnerInteger::U8(value) => Ok(*value as i64),
            InnerInteger::I16(value) => Ok(*value as i64),
            InnerInteger::U16(value) => Ok(*value as i64),
            InnerInteger::I32(value) => Ok(*value as i64),
            InnerInteger::U32(value) => Ok(*value as i64),
            InnerInteger::I64(value) => Ok(*value),
            InnerInteger::U64(value) => {
                if *value <= i64::MAX as u64 {
                    Ok(*value as i64)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            InnerInteger::U128(_) | InnerInteger::I128(_) => Err(Error::ImpreciseCastWouldLoseData),
        }
    }

    /// Returns the contained value as an i64, or an error if the value is unable to fit.
    #[allow(clippy::cast_possible_wrap)]
    #[inline]
    pub const fn as_i128(&self) -> Result<i128, Error> {
        match &self.0 {
            InnerInteger::I8(value) => Ok(*value as i128),
            InnerInteger::U8(value) => Ok(*value as i128),
            InnerInteger::I16(value) => Ok(*value as i128),
            InnerInteger::U16(value) => Ok(*value as i128),
            InnerInteger::I32(value) => Ok(*value as i128),
            InnerInteger::U32(value) => Ok(*value as i128),
            InnerInteger::I64(value) => Ok(*value as i128),
            InnerInteger::U64(value) => Ok(*value as i128),
            InnerInteger::I128(value) => Ok(*value),
            InnerInteger::U128(value) => {
                if *value <= i128::MAX as u128 {
                    Ok(*value as i128)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
        }
    }

    /// Returns the contained value as an u64, or an error if the value is unable to fit.
    #[allow(clippy::cast_sign_loss)]
    #[inline]
    pub const fn as_u64(&self) -> Result<u64, Error> {
        match &self.0 {
            InnerInteger::I8(value) => {
                if *value >= 0 {
                    Ok(*value as u64)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            InnerInteger::U8(value) => Ok(*value as u64),
            InnerInteger::I16(value) => {
                if *value >= 0 {
                    Ok(*value as u64)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            InnerInteger::U16(value) => Ok(*value as u64),
            InnerInteger::U32(value) => Ok(*value as u64),
            InnerInteger::I32(value) => {
                if *value >= 0 {
                    Ok(*value as u64)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            InnerInteger::U64(value) => Ok(*value),
            InnerInteger::I64(value) => {
                if *value >= 0 {
                    Ok(*value as u64)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            InnerInteger::U128(_) | InnerInteger::I128(_) => Err(Error::ImpreciseCastWouldLoseData),
        }
    }

    /// Returns the contained value as an u64, or an error if the value is unable to fit.
    #[allow(clippy::cast_sign_loss)]
    #[inline]
    pub const fn as_u128(&self) -> Result<u128, Error> {
        match &self.0 {
            InnerInteger::I8(value) => {
                if *value >= 0 {
                    Ok(*value as u128)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            InnerInteger::U8(value) => Ok(*value as u128),
            InnerInteger::I16(value) => {
                if *value >= 0 {
                    Ok(*value as u128)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            InnerInteger::U16(value) => Ok(*value as u128),
            InnerInteger::U32(value) => Ok(*value as u128),
            InnerInteger::I32(value) => {
                if *value >= 0 {
                    Ok(*value as u128)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            InnerInteger::U64(value) => Ok(*value as u128),
            InnerInteger::I64(value) => {
                if *value >= 0 {
                    Ok(*value as u128)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            InnerInteger::U128(value) => Ok(*value),
            InnerInteger::I128(value) => {
                if *value >= 0 {
                    Ok(*value as u128)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
        }
    }

    /// Writes this value using the smallest form possible.
    #[inline]
    pub fn write_to<W: WriteBytesExt>(&self, writer: W) -> std::io::Result<usize> {
        match self.0 {
            InnerInteger::I8(value) => write_i8(writer, value),
            InnerInteger::I16(value) => write_i16(writer, value),
            InnerInteger::I32(value) => write_i32(writer, value),
            InnerInteger::I64(value) => write_i64(writer, value),
            InnerInteger::I128(value) => write_i128(writer, value),
            InnerInteger::U8(value) => write_u8(writer, value),
            InnerInteger::U16(value) => write_u16(writer, value),
            InnerInteger::U32(value) => write_u32(writer, value),
            InnerInteger::U64(value) => write_u64(writer, value),
            InnerInteger::U128(value) => write_u128(writer, value),
        }
    }

    /// Reads an integer based on the atom header (`kind` and `byte_len`).
    /// `byte_len` should be the argument from the atom header directly.
    #[inline]
    pub fn read_from<R: ReadBytesExt>(
        kind: Kind,
        byte_len: usize,
        reader: &mut R,
    ) -> Result<Self, Error> {
        match kind {
            Kind::Int => match byte_len {
                1 => Ok(InnerInteger::I8(reader.read_i8()?)),
                2 => Ok(InnerInteger::I16(reader.read_i16::<LittleEndian>()?)),
                3 => Ok(InnerInteger::I32(reader.read_i24::<LittleEndian>()?)),
                4 => Ok(InnerInteger::I32(reader.read_i32::<LittleEndian>()?)),
                6 => Ok(InnerInteger::I64(reader.read_i48::<LittleEndian>()?)),
                8 => Ok(InnerInteger::I64(reader.read_i64::<LittleEndian>()?)),
                16 => Ok(InnerInteger::I128(reader.read_i128::<LittleEndian>()?)),
                count => Err(Error::UnsupportedByteCount(kind, count)),
            },
            Kind::UInt => match byte_len {
                1 => Ok(InnerInteger::U8(reader.read_u8()?)),
                2 => Ok(InnerInteger::U16(reader.read_u16::<LittleEndian>()?)),
                3 => Ok(InnerInteger::U32(reader.read_u24::<LittleEndian>()?)),
                4 => Ok(InnerInteger::U32(reader.read_u32::<LittleEndian>()?)),
                6 => Ok(InnerInteger::U64(reader.read_u48::<LittleEndian>()?)),
                8 => Ok(InnerInteger::U64(reader.read_u64::<LittleEndian>()?)),
                16 => Ok(InnerInteger::U128(reader.read_u128::<LittleEndian>()?)),
                count => Err(Error::UnsupportedByteCount(kind, count)),
            },
            _ => Err(Error::UnexpectedKind(kind, Kind::Int)),
        }
        .map(Integer)
    }

    /// Converts this integer to an f32, but only if it can be done without losing precision.
    #[allow(clippy::cast_precision_loss)]
    #[inline]
    pub fn as_f32(&self) -> Result<f32, Error> {
        let int = self.as_i32()?;
        if int < -(2_i32.pow(f32::MANTISSA_DIGITS)) || int >= 2_i32.pow(f32::MANTISSA_DIGITS) {
            Err(Error::ImpreciseCastWouldLoseData)
        } else {
            Ok(int as f32)
        }
    }

    /// Converts this integer to an f64, but only if it can be done without losing precision.
    #[allow(clippy::cast_precision_loss)]
    #[inline]
    pub fn as_f64(&self) -> Result<f64, Error> {
        let int = self.as_i64()?;
        if int < -(2_i64.pow(f64::MANTISSA_DIGITS)) || int >= 2_i64.pow(f64::MANTISSA_DIGITS) {
            Err(Error::ImpreciseCastWouldLoseData)
        } else {
            Ok(int as f64)
        }
    }

    /// Converts this integer to an f64, but only if it can be done without losing precision.
    #[allow(clippy::cast_precision_loss)]
    #[inline]
    pub fn as_float(&self) -> Result<Float, Error> {
        self.as_f32()
            .map(Float::from)
            .or_else(|_| self.as_f64().map(Float::from))
    }
}

impl From<u8> for Integer {
    #[inline]
    fn from(value: u8) -> Self {
        Self(InnerInteger::U8(value))
    }
}

macro_rules! impl_from_unsigned_integer {
    ($primitive:ty, $smaller_primitive:ty, $variant:ident) => {
        impl From<$primitive> for Integer {
            #[inline]
            fn from(value: $primitive) -> Self {
                if let Ok(value) = <$smaller_primitive>::try_from(value) {
                    Self::from(value as $smaller_primitive)
                } else {
                    Integer(InnerInteger::$variant(value))
                }
            }
        }
    };
}

impl_from_unsigned_integer!(u16, u8, U16);
impl_from_unsigned_integer!(u32, u16, U32);
impl_from_unsigned_integer!(u64, u32, U64);
impl_from_unsigned_integer!(u128, u64, U128);

impl From<i8> for Integer {
    #[inline]
    fn from(value: i8) -> Self {
        Self(InnerInteger::I8(value))
    }
}

macro_rules! impl_from_unsigned_integer {
    ($primitive:ty, $smaller_primitive:ty, $smaller_unsigned_primitive:ty, $variant:ident) => {
        impl From<$primitive> for Integer {
            #[inline]
            fn from(value: $primitive) -> Self {
                if let Ok(value) = <$smaller_primitive>::try_from(value) {
                    Self::from(value as $smaller_primitive)
                } else if let Ok(value) = <$smaller_unsigned_primitive>::try_from(value) {
                    Self::from(value as $smaller_unsigned_primitive)
                } else {
                    Integer(InnerInteger::$variant(value))
                }
            }
        }
    };
}

impl_from_unsigned_integer!(i16, i8, u8, I16);
impl_from_unsigned_integer!(i32, i16, u16, I32);
impl_from_unsigned_integer!(i64, i32, u32, I64);
impl_from_unsigned_integer!(i128, i64, u64, I128);

/// Reads an atom.
#[allow(clippy::cast_possible_truncation)]
#[inline]
pub fn read_atom<'de, R: Reader<'de>>(
    reader: &mut R,
    remaining_budget: &mut usize,
    scratch: &mut Vec<u8>,
) -> Result<Atom<'de>, Error> {
    let (kind, arg) = read_atom_header(reader)?;
    Ok(match kind {
        Kind::Sequence | Kind::Map | Kind::Symbol => Atom {
            kind,
            arg,
            nucleus: None,
        },
        Kind::Special => Atom {
            kind,
            arg,
            nucleus: match Special::try_from(arg)? {
                Special::None => None,
                Special::Unit => Some(Nucleus::Unit),
                Special::False => Some(Nucleus::Boolean(false)),
                Special::True => Some(Nucleus::Boolean(true)),
                Special::Named => Some(Nucleus::Named),
                Special::DynamicMap => Some(Nucleus::DynamicMap),
                Special::DynamicEnd => Some(Nucleus::DynamicEnd),
            },
        },
        Kind::Int | Kind::UInt => {
            let bytes = arg as usize + 1;
            update_budget(remaining_budget, in_memory_int_size(bytes))?;
            Atom {
                kind,
                arg,
                nucleus: Some(Nucleus::Integer(Integer::read_from(kind, bytes, reader)?)),
            }
        }
        Kind::Float => {
            let bytes = arg as usize + 1;
            update_budget(remaining_budget, in_memory_int_size(bytes))?;
            Atom {
                kind,
                arg,
                nucleus: Some(Nucleus::Float(Float::read_from(kind, bytes, reader)?)),
            }
        }
        Kind::Bytes => {
            let bytes = arg as usize;
            update_budget(remaining_budget, bytes)?;
            let bytes = reader.buffered_read_bytes(bytes, scratch)?;
            Atom {
                kind,
                arg,
                nucleus: Some(Nucleus::Bytes(bytes)),
            }
        }
    })
}

#[inline]
pub(crate) const fn in_memory_int_size(encoded_length: usize) -> usize {
    // Some integers are stored more compact than we can represent them in memory
    match encoded_length {
        3 => 4,
        6 => 8,
        other => other,
    }
}

#[inline]
pub(crate) fn update_budget(budget: &mut usize, read_amount: usize) -> Result<(), Error> {
    if let Some(remaining) = budget.checked_sub(read_amount) {
        *budget = remaining;
        Ok(())
    } else {
        Err(Error::TooManyBytesRead)
    }
}

/// An encoded [`Kind`], argument, and optional contained value.
#[derive(Debug)]
pub struct Atom<'de> {
    /// The type of atom.
    pub kind: Kind,
    /// The argument contained in the atom header.
    pub arg: u64,
    /// The contained value, if any.
    pub nucleus: Option<Nucleus<'de>>,
}

/// A floating point number that can safely convert between other number types using compile-time evaluation when possible.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Float(pub(crate) InnerFloat);

#[derive(Debug, Copy, Clone)]
pub(crate) enum InnerFloat {
    /// An f64 value.
    F64(f64),
    /// An f32 value.
    F32(f32),
}

impl From<f32> for Float {
    #[inline]
    fn from(value: f32) -> Self {
        Self(InnerFloat::F32(value))
    }
}

impl From<f64> for Float {
    #[inline]
    fn from(value: f64) -> Self {
        Self(InnerFloat::F64(value))
    }
}

impl PartialEq for InnerFloat {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (InnerFloat::F64(left), InnerFloat::F64(right)) => left == right,
            (InnerFloat::F32(left), InnerFloat::F32(right)) => left == right,
            (InnerFloat::F64(left), InnerFloat::F32(right)) => *left == f64::from(*right),
            (InnerFloat::F32(left), InnerFloat::F64(right)) => f64::from(*left) == *right,
        }
    }
}

impl Display for Float {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            InnerFloat::F32(value) => Display::fmt(value, f),
            InnerFloat::F64(value) => Display::fmt(value, f),
        }
    }
}

impl Float {
    /// Returns true if the value contained is zero.
    #[must_use]
    #[inline]
    pub fn is_zero(&self) -> bool {
        match self.0 {
            InnerFloat::F32(value) => value.abs() <= f32::EPSILON,
            InnerFloat::F64(value) => value.abs() <= f64::EPSILON,
        }
    }

    /// Returns this number as an f32, if it can be done without losing precision.
    #[allow(clippy::float_cmp, clippy::cast_possible_truncation)]
    #[inline]
    pub fn as_f32(&self) -> Result<f32, Error> {
        match self.0 {
            InnerFloat::F32(value) => Ok(value),
            InnerFloat::F64(value) => {
                let converted = value as f32;
                if f64::from(converted) == value {
                    Ok(converted)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
        }
    }

    /// Returns this number as an f64.
    #[must_use]
    #[inline]
    pub const fn as_f64(&self) -> f64 {
        match self.0 {
            InnerFloat::F64(value) => value,
            InnerFloat::F32(value) => value as f64,
        }
    }

    /// Returns this number as an [`Integer`], if the stored value has no fractional part.
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    pub fn as_integer(&self) -> Result<Integer, Error> {
        match self.0 {
            InnerFloat::F64(value) => {
                if value.fract().abs() < f64::EPSILON {
                    // no fraction, safe to convert
                    Ok(Integer::from(value as i64))
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            InnerFloat::F32(value) => {
                if value.fract().abs() < f32::EPSILON {
                    Ok(Integer::from(value as i32))
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
        }
    }

    /// Writes this value using the smallest form possible.
    #[inline]
    pub fn write_to<W: WriteBytesExt>(&self, writer: W) -> std::io::Result<usize> {
        match self.0 {
            InnerFloat::F64(float) => write_f64(writer, float),
            InnerFloat::F32(float) => write_f32(writer, float),
        }
    }

    /// Reads a floating point number given the atom `kind` and `byte_len`.
    /// `byte_len` should be the exact argument from the atom header.
    #[inline]
    pub fn read_from<R: ReadBytesExt>(
        kind: Kind,
        byte_len: usize,
        reader: &mut R,
    ) -> Result<Self, Error> {
        if Kind::Float == kind {
            match byte_len {
                2 => Ok(Self::from(read_f16(reader)?)),
                4 => Ok(Self::from(reader.read_f32::<LittleEndian>()?)),
                8 => Ok(Self::from(reader.read_f64::<LittleEndian>()?)),
                count => Err(Error::UnsupportedByteCount(Kind::Float, count)),
            }
        } else {
            Err(Error::UnexpectedKind(kind, Kind::Float))
        }
    }
}

/// A value contained within an [`Atom`].
#[derive(Debug)]
pub enum Nucleus<'de> {
    /// A boolean value.
    Boolean(bool),
    /// An integer value.
    Integer(Integer),
    /// A floating point value.
    Float(Float),
    /// A buffer of bytes.
    Bytes(BufferedBytes<'de>),
    /// A unit.
    Unit,
    /// A named value.
    Named,
    /// A marker denoting a map with unknown length is next in the file.
    DynamicMap,
    /// A marker denoting the end of a map with unknown length.
    DynamicEnd,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::cast_possible_truncation)]
    fn test_roundtrip_integer(input: Integer, expected: Integer, expected_size: usize) {
        let mut out = Vec::new();
        assert_eq!(input.write_to(&mut out).unwrap(), expected_size);
        {
            let mut reader = out.as_slice();
            let (kind, bytes) = read_atom_header(&mut reader).unwrap();
            assert_eq!(
                Integer::read_from(kind, bytes as usize + 1, &mut reader).unwrap(),
                expected
            );
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn test_roundtrip_float(input: Float, expected: Float, expected_size: usize) {
        let mut out = Vec::new();
        assert_eq!(input.write_to(&mut out).unwrap(), expected_size);
        {
            let mut reader = out.as_slice();
            let (kind, bytes) = read_atom_header(&mut reader).unwrap();
            assert_eq!(
                Float::read_from(kind, bytes as usize + 1, &mut reader).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn header() {
        let mut out = Vec::new();
        write_header(&mut out, 1).unwrap();
        let version = read_header(&mut out.as_slice()).unwrap();
        assert_eq!(version, 1);

        out[0] = 0;
        assert!(read_header(&mut out.as_slice()).is_err());
    }

    #[test]
    fn atom_header_args() {
        let mut out = Vec::new();
        for arg in 1..=64 {
            let arg = 2_u64.saturating_pow(arg);
            write_atom_header(&mut out, Kind::Map, arg).unwrap();
            println!("header: {out:?}");
            let (kind, read_arg) = read_atom_header(&mut out.as_slice()).unwrap();
            assert_eq!(kind, Kind::Map);
            assert_eq!(read_arg, arg);
            out.clear();
        }
    }

    #[test]
    fn atom_kinds() {
        assert_eq!(Kind::Special, Kind::from_u8(Kind::Special as u8).unwrap());
        assert_eq!(Kind::Int, Kind::from_u8(Kind::Int as u8).unwrap());
        assert_eq!(Kind::UInt, Kind::from_u8(Kind::UInt as u8).unwrap());
        assert_eq!(Kind::Float, Kind::from_u8(Kind::Float as u8).unwrap());
        assert_eq!(Kind::Sequence, Kind::from_u8(Kind::Sequence as u8).unwrap());
        assert_eq!(Kind::Map, Kind::from_u8(Kind::Map as u8).unwrap());
        assert_eq!(Kind::Symbol, Kind::from_u8(Kind::Symbol as u8).unwrap());
        assert_eq!(Kind::Bytes, Kind::from_u8(Kind::Bytes as u8).unwrap());
        for i in 8_u8..=15 {
            assert!(Kind::from_u8(i).is_err());
        }
    }

    #[test]
    fn zero() {
        test_roundtrip_integer(Integer::from(0_u64), Integer(InnerInteger::U8(0)), 2);
        test_roundtrip_integer(Integer::from(0_i64), Integer(InnerInteger::I8(0)), 2);
        test_roundtrip_float(Float::from(0_f32), Float(InnerFloat::F32(0.)), 3);
        test_roundtrip_float(Float::from(0_f64), Float(InnerFloat::F32(0.)), 3);
    }

    #[test]
    fn u8_max() {
        test_roundtrip_integer(
            Integer::from(u64::from(u8::MAX)),
            Integer(InnerInteger::U8(u8::MAX)),
            2,
        );
    }

    #[test]
    fn i8_max() {
        test_roundtrip_integer(
            Integer::from(i64::from(i8::MAX)),
            Integer(InnerInteger::I8(i8::MAX)),
            2,
        );
    }

    #[test]
    fn i8_min() {
        test_roundtrip_integer(
            Integer::from(i64::from(i8::MIN)),
            Integer(InnerInteger::I8(i8::MIN)),
            2,
        );
    }

    #[test]
    fn u16_max() {
        test_roundtrip_integer(
            Integer::from(u64::from(u16::MAX)),
            Integer(InnerInteger::U16(u16::MAX)),
            3,
        );
    }

    #[test]
    fn i16_max() {
        test_roundtrip_integer(
            Integer::from(i64::from(i16::MAX)),
            Integer(InnerInteger::I16(i16::MAX)),
            3,
        );
    }

    #[test]
    fn i16_min() {
        test_roundtrip_integer(
            Integer::from(i64::from(i16::MIN)),
            Integer(InnerInteger::I16(i16::MIN)),
            3,
        );
    }

    #[test]
    fn u32_max() {
        test_roundtrip_integer(
            Integer::from(u64::from(u32::MAX)),
            Integer(InnerInteger::U32(u32::MAX)),
            5,
        );
    }

    #[test]
    fn i32_max() {
        test_roundtrip_integer(
            Integer::from(i64::from(i32::MAX)),
            Integer(InnerInteger::I32(i32::MAX)),
            5,
        );
    }

    #[test]
    fn i32_min() {
        test_roundtrip_integer(
            Integer::from(i64::from(i32::MIN)),
            Integer(InnerInteger::I32(i32::MIN)),
            5,
        );
    }

    #[test]
    fn u64_max() {
        test_roundtrip_integer(
            Integer::from(u64::MAX),
            Integer(InnerInteger::U64(u64::MAX)),
            9,
        );
    }

    #[test]
    fn i64_max() {
        test_roundtrip_integer(
            Integer::from(i64::MAX),
            Integer(InnerInteger::I64(i64::MAX)),
            9,
        );
    }

    #[test]
    fn i64_min() {
        test_roundtrip_integer(
            Integer::from(i64::MIN),
            Integer(InnerInteger::I64(i64::MIN)),
            9,
        );
    }

    #[test]
    fn u128_max() {
        test_roundtrip_integer(
            Integer::from(u128::MAX),
            Integer(InnerInteger::U128(u128::MAX)),
            17,
        );
    }

    #[test]
    fn i128_max() {
        test_roundtrip_integer(
            Integer::from(i128::MAX),
            Integer(InnerInteger::I128(i128::MAX)),
            17,
        );
    }

    #[test]
    fn i128_min() {
        test_roundtrip_integer(
            Integer::from(i128::MIN),
            Integer(InnerInteger::I128(i128::MIN)),
            17,
        );
    }

    #[test]
    fn integer_is_zero() {
        assert!(Integer::from(0_i128).is_zero());
        assert!(!Integer::from(i8::MAX).is_zero());
        assert!(!Integer::from(i16::MAX).is_zero());
        assert!(!Integer::from(i32::MAX).is_zero());
        assert!(!Integer::from(i64::MAX).is_zero());
        assert!(!Integer::from(i128::MAX).is_zero());

        assert!(Integer::from(0_u128).is_zero());
        assert!(!Integer::from(u8::MAX).is_zero());
        assert!(!Integer::from(u16::MAX).is_zero());
        assert!(!Integer::from(u32::MAX).is_zero());
        assert!(!Integer::from(u64::MAX).is_zero());
        assert!(!Integer::from(u128::MAX).is_zero());
    }

    macro_rules! test_conversion_succeeds {
        ($host:ty, $value:expr, $method:ident) => {{
            assert!(<$host>::from($value).$method().is_ok())
        }};
    }
    macro_rules! test_conversion_fails {
        ($host:ty, $value:expr, $method:ident) => {{
            assert!(matches!(
                <$host>::from($value).$method(),
                Err(Error::ImpreciseCastWouldLoseData)
            ))
        }};
    }

    #[test]
    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    fn integer_casts() {
        macro_rules! test_negative_fails {
            ($method:ident) => {{
                test_conversion_fails!(Integer, i8::MIN, $method);
                test_conversion_fails!(Integer, i16::MIN, $method);
                test_conversion_fails!(Integer, i32::MIN, $method);
                test_conversion_fails!(Integer, i64::MIN, $method);
                test_conversion_fails!(Integer, i128::MIN, $method);
            }};
        }

        // ### i8 ###
        // unsigned max -> i8
        test_conversion_fails!(Integer, u8::MAX, as_i8);
        test_conversion_fails!(Integer, u16::MAX, as_i8);
        test_conversion_fails!(Integer, u32::MAX, as_i8);
        test_conversion_fails!(Integer, u64::MAX, as_i8);
        test_conversion_fails!(Integer, u128::MAX, as_i8);

        // signed max -> i8
        test_conversion_succeeds!(Integer, i8::MAX, as_i8);
        test_conversion_fails!(Integer, i16::MAX, as_i8);
        test_conversion_fails!(Integer, i32::MAX, as_i8);
        test_conversion_fails!(Integer, i64::MAX, as_i8);
        test_conversion_fails!(Integer, i128::MAX, as_i8);

        // signed max as unsigned -> i8
        test_conversion_succeeds!(Integer, i8::MAX as u8, as_i8);
        test_conversion_fails!(Integer, i16::MAX as u16, as_i8);
        test_conversion_fails!(Integer, i32::MAX as u32, as_i8);
        test_conversion_fails!(Integer, i64::MAX as u64, as_i8);
        test_conversion_fails!(Integer, i128::MAX as u128, as_i8);

        // signed min -> i8
        test_conversion_succeeds!(Integer, i8::MIN, as_i8);
        test_conversion_fails!(Integer, i16::MIN, as_i8);
        test_conversion_fails!(Integer, i32::MIN, as_i8);
        test_conversion_fails!(Integer, i64::MIN, as_i8);
        test_conversion_fails!(Integer, i128::MIN, as_i8);

        // ### i16 ###
        // unsigned max -> i16
        test_conversion_succeeds!(Integer, u8::MAX, as_i16);
        test_conversion_fails!(Integer, u16::MAX, as_i16);
        test_conversion_fails!(Integer, u32::MAX, as_i16);
        test_conversion_fails!(Integer, u64::MAX, as_i16);
        test_conversion_fails!(Integer, u128::MAX, as_i16);

        // signed max -> i16
        test_conversion_succeeds!(Integer, i8::MAX, as_i16);
        test_conversion_succeeds!(Integer, i16::MAX, as_i16);
        test_conversion_fails!(Integer, i32::MAX, as_i16);
        test_conversion_fails!(Integer, i64::MAX, as_i16);
        test_conversion_fails!(Integer, i128::MAX, as_i16);

        // signed max as unsigned -> i16
        test_conversion_succeeds!(Integer, i8::MAX as u8, as_i16);
        test_conversion_succeeds!(Integer, i16::MAX as u16, as_i16);
        test_conversion_fails!(Integer, i32::MAX as u32, as_i16);
        test_conversion_fails!(Integer, i64::MAX as u64, as_i16);
        test_conversion_fails!(Integer, i128::MAX as u128, as_i16);

        // signed min -> i16
        test_conversion_succeeds!(Integer, i8::MIN, as_i16);
        test_conversion_succeeds!(Integer, i16::MIN, as_i16);
        test_conversion_fails!(Integer, i32::MIN, as_i16);
        test_conversion_fails!(Integer, i64::MIN, as_i16);
        test_conversion_fails!(Integer, i128::MIN, as_i16);

        // ### i32 ###
        // unsigned max -> i32
        test_conversion_succeeds!(Integer, u8::MAX, as_i32);
        test_conversion_succeeds!(Integer, u16::MAX, as_i32);
        test_conversion_fails!(Integer, u32::MAX, as_i32);
        test_conversion_fails!(Integer, u64::MAX, as_i32);
        test_conversion_fails!(Integer, u128::MAX, as_i32);

        // signed max -> i32
        test_conversion_succeeds!(Integer, i8::MAX, as_i32);
        test_conversion_succeeds!(Integer, i16::MAX, as_i32);
        test_conversion_succeeds!(Integer, i32::MAX, as_i32);
        test_conversion_fails!(Integer, i64::MAX, as_i32);
        test_conversion_fails!(Integer, i128::MAX, as_i32);

        // signed max as unsigned -> i32
        test_conversion_succeeds!(Integer, i8::MAX as u8, as_i32);
        test_conversion_succeeds!(Integer, i16::MAX as u16, as_i32);
        test_conversion_succeeds!(Integer, i32::MAX as u32, as_i32);
        test_conversion_fails!(Integer, i64::MAX as u64, as_i32);
        test_conversion_fails!(Integer, i128::MAX as u128, as_i32);

        // signed min -> i32
        test_conversion_succeeds!(Integer, i8::MIN, as_i32);
        test_conversion_succeeds!(Integer, i16::MIN, as_i32);
        test_conversion_succeeds!(Integer, i32::MIN, as_i32);
        test_conversion_fails!(Integer, i64::MIN, as_i32);
        test_conversion_fails!(Integer, i128::MIN, as_i32);

        // ### i64 ###
        // unsigned max -> i64
        test_conversion_succeeds!(Integer, u8::MAX, as_i64);
        test_conversion_succeeds!(Integer, u16::MAX, as_i64);
        test_conversion_succeeds!(Integer, u32::MAX, as_i64);
        test_conversion_fails!(Integer, u64::MAX, as_i64);
        test_conversion_fails!(Integer, u128::MAX, as_i64);

        // signed max -> i64
        test_conversion_succeeds!(Integer, i8::MAX, as_i64);
        test_conversion_succeeds!(Integer, i16::MAX, as_i64);
        test_conversion_succeeds!(Integer, i32::MAX, as_i64);
        test_conversion_succeeds!(Integer, i64::MAX, as_i64);
        test_conversion_fails!(Integer, i128::MAX, as_i64);

        // signed max as unsigned -> i64
        test_conversion_succeeds!(Integer, i8::MAX as u8, as_i64);
        test_conversion_succeeds!(Integer, i16::MAX as u16, as_i64);
        test_conversion_succeeds!(Integer, i32::MAX as u32, as_i64);
        test_conversion_succeeds!(Integer, i64::MAX as u64, as_i64);
        test_conversion_fails!(Integer, i128::MAX as u128, as_i64);

        // signed min -> i64
        test_conversion_succeeds!(Integer, i8::MIN, as_i64);
        test_conversion_succeeds!(Integer, i16::MIN, as_i64);
        test_conversion_succeeds!(Integer, i32::MIN, as_i64);
        test_conversion_succeeds!(Integer, i64::MIN, as_i64);
        test_conversion_fails!(Integer, i128::MIN, as_i64);

        // ### i128 ###
        // unsigned max -> i128
        test_conversion_succeeds!(Integer, u8::MAX, as_i128);
        test_conversion_succeeds!(Integer, u16::MAX, as_i128);
        test_conversion_succeeds!(Integer, u32::MAX, as_i128);
        test_conversion_succeeds!(Integer, u64::MAX, as_i128);
        test_conversion_fails!(Integer, u128::MAX, as_i128);

        // signed max -> i128
        test_conversion_succeeds!(Integer, i8::MAX, as_i128);
        test_conversion_succeeds!(Integer, i16::MAX, as_i128);
        test_conversion_succeeds!(Integer, i32::MAX, as_i128);
        test_conversion_succeeds!(Integer, i64::MAX, as_i128);
        test_conversion_succeeds!(Integer, i128::MAX, as_i128);

        // signed max as unsigned -> i128
        test_conversion_succeeds!(Integer, i8::MAX as u8, as_i128);
        test_conversion_succeeds!(Integer, i16::MAX as u16, as_i128);
        test_conversion_succeeds!(Integer, i32::MAX as u32, as_i128);
        test_conversion_succeeds!(Integer, i64::MAX as u64, as_i128);
        test_conversion_succeeds!(Integer, i128::MAX as u128, as_i128);

        // signed min -> i128
        test_conversion_succeeds!(Integer, i8::MIN, as_i128);
        test_conversion_succeeds!(Integer, i16::MIN, as_i128);
        test_conversion_succeeds!(Integer, i32::MIN, as_i128);
        test_conversion_succeeds!(Integer, i64::MIN, as_i128);
        test_conversion_succeeds!(Integer, i128::MIN, as_i128);

        // ### u8 ###
        // unsigned max -> u8
        test_conversion_succeeds!(Integer, u8::MAX, as_u8);
        test_conversion_fails!(Integer, u16::MAX, as_u8);
        test_conversion_fails!(Integer, u32::MAX, as_u8);
        test_conversion_fails!(Integer, u64::MAX, as_u8);
        test_conversion_fails!(Integer, u128::MAX, as_u8);

        // signed max -> u8
        test_conversion_succeeds!(Integer, i8::MAX, as_u8);
        test_conversion_fails!(Integer, i16::MAX, as_u8);
        test_conversion_fails!(Integer, i32::MAX, as_u8);
        test_conversion_fails!(Integer, i64::MAX, as_u8);
        test_conversion_fails!(Integer, i128::MAX, as_u8);

        // signed min -> u8
        test_negative_fails!(as_u8);

        // ### u16 ###
        // unsigned max -> u16
        test_conversion_succeeds!(Integer, u8::MAX, as_u16);
        test_conversion_succeeds!(Integer, u16::MAX, as_u16);
        test_conversion_fails!(Integer, u32::MAX, as_u16);
        test_conversion_fails!(Integer, u64::MAX, as_u16);
        test_conversion_fails!(Integer, u128::MAX, as_u16);

        // signed max -> u16
        test_conversion_succeeds!(Integer, i8::MAX, as_u16);
        test_conversion_succeeds!(Integer, i16::MAX, as_u16);
        test_conversion_fails!(Integer, i32::MAX, as_u16);
        test_conversion_fails!(Integer, i64::MAX, as_u16);
        test_conversion_fails!(Integer, i128::MAX, as_u16);

        // signed min -> u16
        test_negative_fails!(as_u16);

        // ### u32 ###
        // unsigned max -> u32
        test_conversion_succeeds!(Integer, u8::MAX, as_u32);
        test_conversion_succeeds!(Integer, u16::MAX, as_u32);
        test_conversion_succeeds!(Integer, u32::MAX, as_u32);
        test_conversion_fails!(Integer, u64::MAX, as_u32);
        test_conversion_fails!(Integer, u128::MAX, as_u32);

        // signed max -> u32
        test_conversion_succeeds!(Integer, i8::MAX, as_u32);
        test_conversion_succeeds!(Integer, i16::MAX, as_u32);
        test_conversion_succeeds!(Integer, i32::MAX, as_u32);
        test_conversion_fails!(Integer, i64::MAX, as_u32);
        test_conversion_fails!(Integer, i128::MAX, as_u32);

        // signed min -> u32
        test_negative_fails!(as_u32);

        // ### u64 ###
        // unsigned max -> u64
        test_conversion_succeeds!(Integer, u8::MAX, as_u64);
        test_conversion_succeeds!(Integer, u16::MAX, as_u64);
        test_conversion_succeeds!(Integer, u32::MAX, as_u64);
        test_conversion_succeeds!(Integer, u64::MAX, as_u64);
        test_conversion_fails!(Integer, u128::MAX, as_u64);

        // signed max -> u64
        test_conversion_succeeds!(Integer, i8::MAX, as_u64);
        test_conversion_succeeds!(Integer, i16::MAX, as_u64);
        test_conversion_succeeds!(Integer, i32::MAX, as_u64);
        test_conversion_succeeds!(Integer, i64::MAX, as_u64);
        test_conversion_fails!(Integer, i128::MAX, as_u64);

        // signed min -> u64
        test_negative_fails!(as_u64);

        // ### u128 ###
        // unsigned max -> u128
        test_conversion_succeeds!(Integer, u8::MAX, as_u128);
        test_conversion_succeeds!(Integer, u16::MAX, as_u128);
        test_conversion_succeeds!(Integer, u32::MAX, as_u128);
        test_conversion_succeeds!(Integer, u64::MAX, as_u128);
        test_conversion_succeeds!(Integer, u128::MAX, as_u128);

        // signed max -> u128
        test_conversion_succeeds!(Integer, i8::MAX, as_u128);
        test_conversion_succeeds!(Integer, i16::MAX, as_u128);
        test_conversion_succeeds!(Integer, i32::MAX, as_u128);
        test_conversion_succeeds!(Integer, i64::MAX, as_u128);
        test_conversion_succeeds!(Integer, i128::MAX, as_u128);

        // signed min -> u128
        test_negative_fails!(as_u128);
    }

    #[test]
    fn float_as_integer() {
        test_conversion_succeeds!(Float, 1_f32, as_integer);
        test_conversion_succeeds!(Float, 1_f64, as_integer);
        test_conversion_fails!(Float, 1.1_f32, as_integer);
        test_conversion_fails!(Float, 1.1_f64, as_integer);
    }

    #[test]
    fn integer_as_float() {
        test_conversion_succeeds!(Integer, -(2_i32.pow(f32::MANTISSA_DIGITS)), as_f32);
        test_conversion_succeeds!(Integer, 2_i32.pow(f32::MANTISSA_DIGITS) - 1, as_f32);
        test_conversion_fails!(Integer, i32::MIN, as_f32);
        test_conversion_fails!(Integer, i32::MAX, as_f32);
        test_conversion_succeeds!(Integer, -(2_i64.pow(f64::MANTISSA_DIGITS)), as_f64);
        test_conversion_succeeds!(Integer, 2_i64.pow(f64::MANTISSA_DIGITS) - 1, as_f64);
        test_conversion_fails!(Integer, i64::MIN, as_f64);
        test_conversion_fails!(Integer, i64::MAX, as_f64);
    }

    #[test]
    fn float_partial_eqs() {
        assert_eq!(Float::from(0_f32), Float::from(0_f32));
        assert_eq!(Float::from(0_f32), Float::from(0_f64));
        assert_eq!(Float::from(0_f64), Float::from(0_f32));
    }

    #[test]
    fn float_is_zero() {
        assert!(Float::from(0_f32).is_zero());
        assert!(!Float::from(1_f32).is_zero());
        assert!(Float::from(0_f64).is_zero());
        assert!(!Float::from(1_f64).is_zero());
    }

    #[test]
    fn integer_display() {
        assert_eq!(Integer::from(i8::MAX).to_string(), "127");
        assert_eq!(Integer::from(i16::MAX).to_string(), "32767");
        assert_eq!(Integer::from(i32::MAX).to_string(), "2147483647");
        assert_eq!(Integer::from(i64::MAX).to_string(), "9223372036854775807");
        assert_eq!(
            Integer::from(i128::MAX).to_string(),
            "170141183460469231731687303715884105727"
        );
        assert_eq!(Integer::from(u8::MAX).to_string(), "255");
        assert_eq!(Integer::from(u16::MAX).to_string(), "65535");
        assert_eq!(Integer::from(u32::MAX).to_string(), "4294967295");
        assert_eq!(Integer::from(u64::MAX).to_string(), "18446744073709551615");
        assert_eq!(
            Integer::from(u128::MAX).to_string(),
            "340282366920938463463374607431768211455"
        );
    }
}
