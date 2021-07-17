use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

pub(crate) const CURRENT_VERSION: u8 = 0;

use crate::{reader::Reader, Error};

/// Writes an atom header into `writer`.
#[allow(clippy::cast_possible_truncation)]
pub fn write_atom_header<W: WriteBytesExt>(
    writer: &mut W,
    kind: Kind,
    arg: Option<u64>,
) -> std::io::Result<usize> {
    // 8 + 64 total bits, with 4 in the first u8, and 7 in the remaining. Total
    // of 10 bytes required. to store a u64::MAX.
    let mut bytes = [0_u8; 10];
    // Kind is the 3 bits.
    let mut first_byte = (kind as u8) << 5;
    let mut arg = arg.unwrap_or_default();
    if arg > 0 {
        // The last 4 bits are the first 4 bits of the arg
        first_byte |= arg as u8 & 0b1111;
        arg >>= 4;
        // If the arg requires more than 4 bits, set the 5th bit.
        if arg > 0 {
            first_byte |= 0b10000;
        }
    }
    bytes[0] = first_byte;
    let mut length = 1;
    while arg > 0 {
        let mut byte = arg as u8 & 0x7F;
        arg >>= 7;
        if arg > 0 {
            byte |= 0b1000_0000;
        }
        bytes[length] = byte;
        length += 1;
    }

    writer.write_all(&bytes[..length])?;

    Ok(length)
}

/// Reads an atom header (kind and argument).
pub fn read_atom_header<R: ReadBytesExt>(reader: &mut R) -> Result<(Kind, u64), Error> {
    let first_byte = reader.read_u8()?;
    let kind = Kind::from_u8(first_byte >> 5)?;
    let mut arg = u64::from(first_byte & 0b1111);
    if first_byte & 0b10000 > 0 {
        let mut bytes_read = 1;
        let mut offset = 4;
        loop {
            let byte = reader.read_u8()?;
            bytes_read += 1;
            arg |= u64::from(byte & 0x7f) << offset;
            offset += 7;
            if byte & 0b1000_0000 == 0 {
                break;
            } else if bytes_read == 9 {
                return Err(Error::InvalidData);
            }
        }
    }

    Ok((kind, arg))
}

/// The type of an atom.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Kind {
    /// No value
    None = 0,
    /// A signed integer. Argument is the byte length, minus one. The following
    /// bytes are the value, stored in little endian.
    Int = 1,
    /// An unsigned integer. Argument is the byte length, minus one. The
    /// following bytes are the value, stored in little endian.
    UInt = 2,
    /// A floating point value. Argument is the byte length, minus one. Must be
    /// either 4 or 8 bytes. The following bytes are the value, stored in little
    /// endian.
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
            0 => Ok(Self::None),
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

/// Writes the Pot header. A u32 written in big endian. The first three bytes
/// are 'Pot' (`0x506F74`), and the fourth byte is the version. The first
/// version of Pot is 0.
pub fn write_header<W: WriteBytesExt>(writer: &mut W, version: u8) -> std::io::Result<usize> {
    writer.write_u32::<BigEndian>(0x506F_7400 | u32::from(version))?;
    Ok(4)
}

#[allow(clippy::similar_names, clippy::cast_possible_truncation)]
pub fn read_header<R: ReadBytesExt>(reader: &mut R) -> Result<u8, Error> {
    let header = reader.read_u32::<BigEndian>()?;
    if header & 0x506F_7400 == 0x506F_7400 {
        let version = (header & 0xFF) as u8;
        Ok(version)
    } else {
        Err(Error::InvalidData)
    }
}

/// Writes a [`Kind::None`] atom.
pub fn write_none<W: WriteBytesExt>(writer: &mut W) -> std::io::Result<usize> {
    write_atom_header(writer, Kind::None, None)
}

/// Writes a [`Kind::None`] atom. Pot doesn't distinguish between a Unit and a None value.
pub fn write_unit<W: WriteBytesExt>(writer: &mut W) -> std::io::Result<usize> {
    write_none(writer)
}

/// Writes an [`Kind::Int`] atom with the given value.
pub fn write_i8<W: WriteBytesExt>(writer: &mut W, value: i8) -> std::io::Result<usize> {
    let header_len = write_atom_header(
        writer,
        Kind::Int,
        Some(std::mem::size_of::<i8>() as u64 - 1),
    )?;
    writer
        .write_i8(value)
        .map(|_| std::mem::size_of::<i8>() + header_len)
}

/// Writes an [`Kind::UInt`] atom with the given value.
pub fn write_u8<W: WriteBytesExt>(writer: &mut W, value: u8) -> std::io::Result<usize> {
    let header_len = write_atom_header(
        writer,
        Kind::UInt,
        Some(std::mem::size_of::<u8>() as u64 - 1),
    )?;
    writer
        .write_u8(value)
        .map(|_| std::mem::size_of::<u8>() + header_len)
}

/// Writes an [`Kind::Float`] atom with the given value.
pub fn write_f32<W: WriteBytesExt>(writer: &mut W, value: f32) -> std::io::Result<usize> {
    let header_len = write_atom_header(
        writer,
        Kind::Float,
        Some(std::mem::size_of::<f32>() as u64 - 1),
    )?;
    writer
        .write_f32::<LittleEndian>(value)
        .map(|_| std::mem::size_of::<f32>() + header_len)
}

/// Writes an [`Kind::Float`] atom with the given value.
pub fn write_f64<W: WriteBytesExt>(writer: &mut W, value: f64) -> std::io::Result<usize> {
    let header_len = write_atom_header(
        writer,
        Kind::Float,
        Some(std::mem::size_of::<f64>() as u64 - 1),
    )?;
    writer
        .write_f64::<LittleEndian>(value)
        .map(|_| std::mem::size_of::<f64>() + header_len)
}

macro_rules! define_integer_write_fn {
    ($kind:expr, $type:ident, $write_fn:tt, $smaller_type:ident, $smaller_write_fn:ident, $docs:literal) => {
        #[doc = $docs]
        pub fn $write_fn<W: WriteBytesExt>(writer: &mut W, value: $type) -> std::io::Result<usize> {
            if let Ok(value) = <$smaller_type as std::convert::TryFrom<$type>>::try_from(value) {
                $smaller_write_fn(writer, value as $smaller_type)
            } else {
                let header_len = write_atom_header(
                    writer,
                    $kind,
                    Some(std::mem::size_of::<$type>() as u64 - 1),
                )?;
                writer
                    .$write_fn::<LittleEndian>(value)
                    .map(|_| std::mem::size_of::<$type>() + header_len)
            }
        }
    };
}

define_integer_write_fn!(Kind::Int, i16, write_i16, i8, write_i8, "Writes an [`Kind::Int`] atom with the given value. Will attempt to write a smaller value if possible to do so.");
define_integer_write_fn!(Kind::UInt, u16, write_u16, u8, write_u8, "Writes an [`Kind::UInt`] atom with the given value. Will attempt to write a smaller value if possible to do so.");
define_integer_write_fn!(Kind::Int, i32, write_i32, i16, write_i16, "Writes an [`Kind::Int`] atom with the given value. Will attempt to write a smaller value if possible to do so.");
define_integer_write_fn!(Kind::UInt, u32, write_u32, u16, write_u16, "Writes an [`Kind::UInt`] atom with the given value. Will attempt to write a smaller value if possible to do so.");
define_integer_write_fn!(Kind::Int, i64, write_i64, i32, write_i32, "Writes an [`Kind::Int`] atom with the given value. Will attempt to write a smaller value if possible to do so.");
define_integer_write_fn!(Kind::UInt, u64, write_u64, u32, write_u32, "Writes an [`Kind::UInt`] atom with the given value. Will attempt to write a smaller value if possible to do so.");

/// Writes an [`Kind::Bytes`] atom with the bytes of the string.
pub fn write_str<W: WriteBytesExt>(writer: &mut W, value: &str) -> std::io::Result<usize> {
    write_bytes(writer, value.as_bytes())
}

/// Writes an [`Kind::Bytes`] atom with the given value.
pub fn write_bytes<W: WriteBytesExt>(writer: &mut W, value: &[u8]) -> std::io::Result<usize> {
    let header_len = write_atom_header(writer, Kind::Bytes, Some(value.len() as u64))?;
    writer.write_all(value)?;
    Ok(value.len() + header_len)
}

/// An integer type that can safely convert between other number types using compile-time evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Integer {
    /// An i8 value.
    I8(i8),
    /// An i16 value.
    I16(i16),
    /// An i32 value.
    I32(i32),
    /// An i64 value.
    I64(i64),
    /// An u8 value.
    U8(u8),
    /// An u16 value.
    U16(u16),
    /// An u32 value.
    U32(u32),
    /// An u64 value.
    U64(u64),
}

impl Integer {
    /// Returns the contained value as an i8, or an error if the value is unable to fit.
    #[allow(clippy::cast_possible_wrap)]
    pub const fn as_i8(&self) -> Result<i8, Error> {
        match self {
            Self::I8(value) => Ok(*value),
            Self::U8(value) => {
                if *value < i8::MAX as u8 {
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
    pub const fn as_u8(&self) -> Result<u8, Error> {
        match self {
            Self::U8(value) => Ok(*value),
            Self::I8(value) => {
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
    pub const fn as_i16(&self) -> Result<i16, Error> {
        match self {
            Self::I8(value) => Ok(*value as i16),
            Self::U8(value) => Ok(*value as i16),
            Self::I16(value) => Ok(*value),
            Self::U16(value) => {
                if *value < i16::MAX as u16 {
                    Ok(*value as i16)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            Self::U32(_) | Self::I32(_) | Self::U64(_) | Self::I64(_) => {
                Err(Error::ImpreciseCastWouldLoseData)
            }
        }
    }

    /// Returns the contained value as an u16, or an error if the value is unable to fit.
    #[allow(clippy::cast_sign_loss)]
    pub const fn as_u16(&self) -> Result<u16, Error> {
        match self {
            Self::I8(value) => {
                if *value >= 0 {
                    Ok(*value as u16)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            Self::U8(value) => Ok(*value as u16),
            Self::U16(value) => Ok(*value),
            Self::I16(value) => {
                if *value >= 0 {
                    Ok(*value as u16)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            Self::U32(_) | Self::I32(_) | Self::U64(_) | Self::I64(_) => {
                Err(Error::ImpreciseCastWouldLoseData)
            }
        }
    }

    /// Returns the contained value as an i32, or an error if the value is unable to fit.
    #[allow(clippy::cast_possible_wrap)]
    pub const fn as_i32(&self) -> Result<i32, Error> {
        match self {
            Self::I8(value) => Ok(*value as i32),
            Self::U8(value) => Ok(*value as i32),
            Self::I16(value) => Ok(*value as i32),
            Self::U16(value) => Ok(*value as i32),
            Self::I32(value) => Ok(*value),
            Self::U32(value) => {
                if *value < i32::MAX as u32 {
                    Ok(*value as i32)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            Self::U64(_) | Self::I64(_) => Err(Error::ImpreciseCastWouldLoseData),
        }
    }

    /// Returns the contained value as an u32, or an error if the value is unable to fit.
    #[allow(clippy::cast_sign_loss)]
    pub const fn as_u32(&self) -> Result<u32, Error> {
        match self {
            Self::I8(value) => {
                if *value >= 0 {
                    Ok(*value as u32)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            Self::U8(value) => Ok(*value as u32),
            Self::I16(value) => {
                if *value >= 0 {
                    Ok(*value as u32)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            Self::U16(value) => Ok(*value as u32),
            Self::U32(value) => Ok(*value),
            Self::I32(value) => {
                if *value >= 0 {
                    Ok(*value as u32)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            Self::U64(_) | Self::I64(_) => Err(Error::ImpreciseCastWouldLoseData),
        }
    }

    /// Returns the contained value as an i64, or an error if the value is unable to fit.
    #[allow(clippy::cast_possible_wrap)]
    pub const fn as_i64(&self) -> Result<i64, Error> {
        match self {
            Self::I8(value) => Ok(*value as i64),
            Self::U8(value) => Ok(*value as i64),
            Self::I16(value) => Ok(*value as i64),
            Self::U16(value) => Ok(*value as i64),
            Self::I32(value) => Ok(*value as i64),
            Self::U32(value) => Ok(*value as i64),
            Self::I64(value) => Ok(*value),
            Self::U64(value) => {
                if *value < i64::MAX as u64 {
                    Ok(*value as i64)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
        }
    }

    /// Returns the contained value as an u64, or an error if the value is unable to fit.
    #[allow(clippy::cast_sign_loss)]
    pub const fn as_u64(&self) -> Result<u64, Error> {
        match self {
            Self::I8(value) => {
                if *value >= 0 {
                    Ok(*value as u64)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            Self::U8(value) => Ok(*value as u64),
            Self::I16(value) => {
                if *value >= 0 {
                    Ok(*value as u64)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            Self::U16(value) => Ok(*value as u64),
            Self::U32(value) => Ok(*value as u64),
            Self::I32(value) => {
                if *value >= 0 {
                    Ok(*value as u64)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            Self::U64(value) => Ok(*value),
            Self::I64(value) => {
                if *value >= 0 {
                    Ok(*value as u64)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
        }
    }

    #[cfg(test)]
    fn write_to<W: WriteBytesExt>(&self, writer: &mut W) -> std::io::Result<usize> {
        match *self {
            Self::I8(value) => write_i8(writer, value),
            Self::I16(value) => write_i16(writer, value),
            Self::I32(value) => write_i32(writer, value),
            Self::I64(value) => write_i64(writer, value),
            Self::U8(value) => write_u8(writer, value),
            Self::U16(value) => write_u16(writer, value),
            Self::U32(value) => write_u32(writer, value),
            Self::U64(value) => write_u64(writer, value),
        }
    }

    /// Reads an integer based on the atom header (`kind` and `byte_len`).
    /// `byte_len` should be the argument from the atom header directly.
    pub fn read_from<R: ReadBytesExt>(
        kind: Kind,
        byte_len: usize,
        reader: &mut R,
    ) -> Result<Self, Error> {
        match kind {
            Kind::None => Ok(Self::U8(0)),
            Kind::Int => match byte_len + 1 {
                1 => Ok(Self::I8(reader.read_i8()?)),
                2 => Ok(Self::I16(reader.read_i16::<LittleEndian>()?)),
                3 => Ok(Self::I32(reader.read_i24::<LittleEndian>()?)),
                4 => Ok(Self::I32(reader.read_i32::<LittleEndian>()?)),
                6 => Ok(Self::I64(reader.read_i48::<LittleEndian>()?)),
                8 => Ok(Self::I64(reader.read_i64::<LittleEndian>()?)),
                _ => Err(Error::InvalidData),
            },
            Kind::UInt => match byte_len + 1 {
                1 => Ok(Self::U8(reader.read_u8()?)),
                2 => Ok(Self::U16(reader.read_u16::<LittleEndian>()?)),
                3 => Ok(Self::U32(reader.read_u24::<LittleEndian>()?)),
                4 => Ok(Self::U32(reader.read_u32::<LittleEndian>()?)),
                6 => Ok(Self::U64(reader.read_u48::<LittleEndian>()?)),
                8 => Ok(Self::U64(reader.read_u64::<LittleEndian>()?)),
                _ => Err(Error::InvalidData),
            },
            _ => Err(Error::InvalidData),
        }
    }

    /// Converts this integer to an f32, but only if it can be done without losing precision.
    #[allow(clippy::cast_precision_loss)]
    pub fn as_f32(&self) -> Result<f32, Error> {
        let int = self.as_i32()?;
        if int < -(2_i32.pow(f32::MANTISSA_DIGITS)) || int > 2_i32.pow(f32::MANTISSA_DIGITS) {
            Err(Error::ImpreciseCastWouldLoseData)
        } else {
            Ok(int as f32)
        }
    }

    /// Converts this integer to an f64, but only if it can be done without losing precision.
    #[allow(clippy::cast_precision_loss)]
    pub fn as_f64(&self) -> Result<f64, Error> {
        let int = self.as_i64()?;
        if int < -(2_i64.pow(f64::MANTISSA_DIGITS)) || int > 2_i64.pow(f64::MANTISSA_DIGITS) {
            Err(Error::ImpreciseCastWouldLoseData)
        } else {
            Ok(int as f64)
        }
    }
}

/// Reads an atom.
#[allow(clippy::cast_possible_truncation)]
pub fn read_atom<'de, R: Reader<'de>>(reader: &mut R) -> Result<Atom<'de>, Error> {
    let (kind, arg) = read_atom_header(reader)?;
    Ok(match kind {
        Kind::None | Kind::Sequence | Kind::Map | Kind::Symbol => Atom {
            kind,
            arg,
            nucleus: None,
        },
        Kind::Int | Kind::UInt => Atom {
            kind,
            arg,
            nucleus: Some(Nucleus::Integer(Integer::read_from(
                kind,
                arg as usize,
                reader,
            )?)),
        },
        Kind::Float => Atom {
            kind,
            arg,
            nucleus: Some(Nucleus::Float(Float::read_from(
                kind,
                arg as usize,
                reader,
            )?)),
        },
        Kind::Bytes => {
            let bytes = reader.buffered_read_bytes(arg as usize)?;
            Atom {
                kind,
                arg,
                nucleus: Some(Nucleus::Bytes(bytes)),
            }
        }
    })
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
#[derive(Debug)]
pub enum Float {
    /// An f64 value.
    F64(f64),
    /// An f32 value.
    F32(f32),
}

impl Float {
    /// Returns this number as an f32, if it can be done without losing precision.
    #[allow(clippy::float_cmp, clippy::cast_possible_truncation)]
    pub fn as_f32(&self) -> Result<f32, Error> {
        match self {
            Self::F32(value) => Ok(*value),
            Self::F64(value) => {
                let converted = *value as f32;
                if f64::from(converted) == *value {
                    Ok(converted)
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
        }
    }

    /// Returns this number as an f64.
    #[must_use]
    pub const fn as_f64(&self) -> f64 {
        match self {
            Self::F64(value) => *value,
            Self::F32(value) => *value as f64,
        }
    }

    /// Returns this number as an [`Integer`], if the stored value has no fractional part.
    #[allow(clippy::cast_possible_truncation)]
    pub fn as_integer(&self) -> Result<Integer, Error> {
        match self {
            Self::F64(value) => {
                if value.fract().abs() < f64::EPSILON {
                    // no fraction, safe to convert
                    Ok(Integer::I64(*value as i64))
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
            Self::F32(value) => {
                if value.fract().abs() < f32::EPSILON {
                    Ok(Integer::I32(*value as i32))
                } else {
                    Err(Error::ImpreciseCastWouldLoseData)
                }
            }
        }
    }

    /// Reads a floating point number given the atom `kind` and `byte_len`.
    /// `byte_len` should be the exact argument from the atom header.
    pub fn read_from<R: ReadBytesExt>(
        kind: Kind,
        byte_len: usize,
        reader: &mut R,
    ) -> Result<Self, Error> {
        if let Kind::Float = kind {
            match byte_len + 1 {
                4 => Ok(Self::F32(reader.read_f32::<LittleEndian>()?)),
                8 => Ok(Self::F64(reader.read_f64::<LittleEndian>()?)),
                _ => Err(Error::InvalidData),
            }
        } else {
            Err(Error::InvalidData)
        }
    }
}

/// A value contained within an [`Atom`].
#[derive(Debug)]
pub enum Nucleus<'de> {
    /// An integer value.
    Integer(Integer),
    /// A floating point value.
    Float(Float),
    /// A buffer of bytes.
    Bytes(&'de [u8]),
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
                Integer::read_from(kind, bytes as usize, &mut reader).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn headers() {
        let mut out = Vec::new();
        write_atom_header(&mut out, Kind::Map, Some(32)).unwrap();
        let (kind, bytes) = read_atom_header(&mut out.as_slice()).unwrap();
        assert_eq!(kind, Kind::Map);
        assert_eq!(bytes, 32);
    }

    #[test]
    fn zero() {
        test_roundtrip_integer(Integer::U64(0), Integer::U8(0), 2);
        test_roundtrip_integer(Integer::I64(0), Integer::I8(0), 2)
    }

    #[test]
    fn u8_max() {
        test_roundtrip_integer(Integer::U64(u64::from(u8::MAX)), Integer::U8(u8::MAX), 2)
    }

    #[test]
    fn i8_max() {
        test_roundtrip_integer(Integer::I64(i64::from(i8::MAX)), Integer::I8(i8::MAX), 2)
    }

    #[test]
    fn i8_min() {
        test_roundtrip_integer(Integer::I64(i64::from(i8::MIN)), Integer::I8(i8::MIN), 2)
    }

    #[test]
    fn u16_max() {
        test_roundtrip_integer(Integer::U64(u64::from(u16::MAX)), Integer::U16(u16::MAX), 3)
    }

    #[test]
    fn i16_max() {
        test_roundtrip_integer(Integer::I64(i64::from(i16::MAX)), Integer::I16(i16::MAX), 3)
    }

    #[test]
    fn i16_min() {
        test_roundtrip_integer(Integer::I64(i64::from(i16::MIN)), Integer::I16(i16::MIN), 3)
    }

    #[test]
    fn u32_max() {
        test_roundtrip_integer(Integer::U64(u64::from(u32::MAX)), Integer::U32(u32::MAX), 5)
    }

    #[test]
    fn i32_max() {
        test_roundtrip_integer(Integer::I64(i64::from(i32::MAX)), Integer::I32(i32::MAX), 5)
    }

    #[test]
    fn i32_min() {
        test_roundtrip_integer(Integer::I64(i64::from(i32::MIN)), Integer::I32(i32::MIN), 5)
    }

    #[test]
    fn u64_max() {
        test_roundtrip_integer(Integer::U64(u64::MAX), Integer::U64(u64::MAX), 9)
    }

    #[test]
    fn i64_max() {
        test_roundtrip_integer(Integer::I64(i64::MAX), Integer::I64(i64::MAX), 9)
    }

    #[test]
    fn i64_min() {
        test_roundtrip_integer(Integer::I64(i64::MIN), Integer::I64(i64::MIN), 9)
    }
}
