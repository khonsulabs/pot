use std::collections::VecDeque;
use std::fmt::Debug;
use std::ops::{Deref, Range};

use byteorder::ReadBytesExt;
use format::Kind;
use serde::de::{
    self, DeserializeSeed, EnumAccess, Error as _, MapAccess, SeqAccess, VariantAccess, Visitor,
};
#[cfg(feature = "tracing")]
use tracing::instrument;

use crate::format::{
    self, Atom, Float, InnerFloat, InnerInteger, Integer, Nucleus, CURRENT_VERSION,
};
use crate::reader::{BufferedBytes, IoReader, Reader, SliceReader};
use crate::{Error, Result};

/// Deserializer for the Pot format.
pub struct Deserializer<'s, 'de, R: Reader<'de>> {
    input: R,
    symbols: SymbolMap<'s, 'de>,
    peeked_atom: VecDeque<Atom<'de>>,
    remaining_budget: usize,
    scratch: Vec<u8>,
}

impl<'s, 'de, R: Reader<'de>> Debug for Deserializer<'s, 'de, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Deserializer")
            .field("symbols", &self.symbols)
            .field("peeked_atom", &self.peeked_atom)
            .field("remaining_budget", &self.remaining_budget)
            .finish()
    }
}

impl<'s, 'de> Deserializer<'s, 'de, SliceReader<'de>> {
    /// Returns a new deserializer for `input`.
    #[inline]
    pub(crate) fn from_slice(input: &'de [u8], maximum_bytes_allocatable: usize) -> Result<Self> {
        Self::from_slice_with_symbols(input, SymbolMap::new(), maximum_bytes_allocatable)
    }

    fn from_slice_with_symbols(
        input: &'de [u8],
        symbols: SymbolMap<'s, 'de>,
        maximum_bytes_allocatable: usize,
    ) -> Result<Self> {
        Self::new(SliceReader::from(input), symbols, maximum_bytes_allocatable)
    }

    /// Returns `true` if the input has been consumed completely.
    #[must_use]
    #[inline]
    pub fn end_of_input(&self) -> bool {
        self.input.data.is_empty() && self.peeked_atom.is_empty()
    }
}

impl<'s, 'de, R: ReadBytesExt> Deserializer<'s, 'de, IoReader<R>> {
    /// Returns a new deserializer for `input`.
    #[inline]
    pub(crate) fn from_read(input: R, maximum_bytes_allocatable: usize) -> Result<Self> {
        Self::new(
            IoReader::new(input),
            SymbolMap::new(),
            maximum_bytes_allocatable,
        )
    }
}

impl<'s, 'de, R: Reader<'de>> Deserializer<'s, 'de, R> {
    #[inline]
    pub(crate) fn new(
        input: R,
        symbols: SymbolMap<'s, 'de>,
        maximum_bytes_allocatable: usize,
    ) -> Result<Self> {
        let mut deserializer = Deserializer {
            input,
            symbols,
            peeked_atom: VecDeque::new(),
            remaining_budget: maximum_bytes_allocatable,
            scratch: Vec::new(),
        };
        deserializer.read_header()?;
        Ok(deserializer)
    }

    fn read_header(&mut self) -> Result<()> {
        let version = format::read_header(&mut self.input)?;
        if version == CURRENT_VERSION {
            Ok(())
        } else {
            Err(Error::IncompatibleVersion)
        }
    }

    fn read_atom(&mut self) -> Result<Atom<'de>> {
        if let Some(peeked) = self.peeked_atom.pop_front() {
            Ok(peeked)
        } else {
            format::read_atom(
                &mut self.input,
                &mut self.remaining_budget,
                &mut self.scratch,
            )
        }
    }

    #[allow(clippy::missing_panics_doc)]
    fn peek_atom_at(&mut self, index: usize) -> Result<&Atom<'_>> {
        while index >= self.peeked_atom.len() {
            let atom = self.read_atom()?;
            self.peeked_atom.push_back(atom);
        }

        Ok(&self.peeked_atom[index])
    }

    #[allow(clippy::missing_panics_doc)]
    fn peek_atom(&mut self) -> Result<&Atom<'_>> {
        self.peek_atom_at(0)
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[allow(clippy::cast_possible_truncation)]
    fn visit_symbol<V>(&mut self, atom: &Atom<'_>, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let is_id = atom.arg & 0b1 != 0;
        let arg = atom.arg >> 1;
        if is_id {
            self.symbols.visit_symbol_id(arg, visitor)
        } else {
            // New symbol
            let name = self
                .input
                .buffered_read_bytes(arg as usize, &mut self.scratch)?;
            match name {
                BufferedBytes::Data(name) => {
                    let name = std::str::from_utf8(name)?;
                    self.symbols.push_borrowed(name);
                    visitor.visit_borrowed_str(name)
                }
                BufferedBytes::Scratch => {
                    let name = std::str::from_utf8(&self.scratch)?;
                    let result = visitor.visit_str(name);
                    self.symbols.push(name);
                    result
                }
            }
        }
    }
}

impl<'a, 'de, 's, R: Reader<'de>> de::Deserializer<'de> for &'a mut Deserializer<'s, 'de, R> {
    type Error = Error;

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;

        match atom.kind {
            Kind::Special => match &atom.nucleus {
                Some(Nucleus::Boolean(value)) => visitor.visit_bool(*value),
                Some(Nucleus::Unit) => visitor.visit_unit(),
                Some(Nucleus::Named) => visitor.visit_map(AtomList::new(self, Some(1))),
                Some(Nucleus::DynamicMap) => visitor.visit_map(AtomList::new(self, None)),
                Some(Nucleus::DynamicEnd) => Err(Error::custom("unexpected dynamic end")),
                Some(Nucleus::Bytes(_) | Nucleus::Integer(_) | Nucleus::Float(_)) => {
                    unreachable!("read_atom can't return this nucleus as a Special")
                }
                None => visitor.visit_none(),
            },
            Kind::Int => match atom.nucleus {
                Some(Nucleus::Integer(Integer(InnerInteger::I8(value)))) => visitor.visit_i8(value),
                Some(Nucleus::Integer(Integer(InnerInteger::I16(value)))) => {
                    visitor.visit_i16(value)
                }
                Some(Nucleus::Integer(Integer(InnerInteger::I32(value)))) => {
                    visitor.visit_i32(value)
                }
                Some(Nucleus::Integer(Integer(InnerInteger::I64(value)))) => {
                    visitor.visit_i64(value)
                }
                Some(Nucleus::Integer(Integer(InnerInteger::I128(value)))) => {
                    visitor.visit_i128(value)
                }
                _ => unreachable!("read_atom should never return anything else"),
            },
            Kind::UInt => match atom.nucleus {
                Some(Nucleus::Integer(Integer(InnerInteger::U8(value)))) => visitor.visit_u8(value),
                Some(Nucleus::Integer(Integer(InnerInteger::U16(value)))) => {
                    visitor.visit_u16(value)
                }
                Some(Nucleus::Integer(Integer(InnerInteger::U32(value)))) => {
                    visitor.visit_u32(value)
                }
                Some(Nucleus::Integer(Integer(InnerInteger::U64(value)))) => {
                    visitor.visit_u64(value)
                }
                Some(Nucleus::Integer(Integer(InnerInteger::U128(value)))) => {
                    visitor.visit_u128(value)
                }
                _ => unreachable!("read_atom should never return anything else"),
            },
            Kind::Float => match atom.nucleus {
                Some(Nucleus::Float(Float(InnerFloat::F32(value)))) => visitor.visit_f32(value),
                Some(Nucleus::Float(Float(InnerFloat::F64(value)))) => visitor.visit_f64(value),
                _ => unreachable!("read_atom should never return anything else"),
            },
            Kind::Sequence => visitor.visit_seq(AtomList::new(self, Some(atom.arg as usize))),
            Kind::Map => visitor.visit_map(AtomList::new(self, Some(atom.arg as usize))),
            Kind::Symbol => self.visit_symbol(&atom, visitor),
            Kind::Bytes => match &atom.nucleus {
                Some(Nucleus::Bytes(bytes)) => match bytes {
                    BufferedBytes::Data(bytes) => {
                        if let Ok(as_str) = std::str::from_utf8(bytes) {
                            visitor.visit_borrowed_str(as_str)
                        } else {
                            visitor.visit_borrowed_bytes(bytes)
                        }
                    }
                    BufferedBytes::Scratch => {
                        if let Ok(as_str) = std::str::from_utf8(&self.scratch) {
                            visitor.visit_str(as_str)
                        } else {
                            visitor.visit_bytes(&self.scratch)
                        }
                    }
                },
                None => visitor.visit_none(),
                // The parsing operation guarantees that this will always be bytes.
                _ => unreachable!(),
            },
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::Special | Kind::UInt | Kind::Int => match atom.nucleus {
                Some(Nucleus::Integer(integer)) => visitor.visit_bool(!integer.is_zero()),
                Some(Nucleus::Boolean(b)) => visitor.visit_bool(b),
                Some(Nucleus::Unit) | None => visitor.visit_bool(false),
                other => Err(Error::custom(format!(
                    "expected bool nucleus, got {other:?}"
                ))),
            },
            other => Err(Error::custom(format!("expected bool, got {other:?}"))),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_i8(integer.as_i8()?)
                } else {
                    unreachable!("read_atom should never return anything else")
                }
            }
            Kind::Special if matches!(atom.nucleus, Some(Nucleus::Unit) | None) => {
                visitor.visit_i8(0)
            }
            other => Err(Error::custom(format!("expected i8, got {other:?}"))),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_i16(integer.as_i16()?)
                } else {
                    unreachable!("read_atom should never return anything else")
                }
            }
            Kind::Special if matches!(atom.nucleus, Some(Nucleus::Unit) | None) => {
                visitor.visit_i16(0)
            }
            other => Err(Error::custom(format!("expected i16, got {other:?}"))),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_i32(integer.as_i32()?)
                } else {
                    unreachable!("read_atom should never return anything else")
                }
            }
            Kind::Special if matches!(atom.nucleus, Some(Nucleus::Unit) | None) => {
                visitor.visit_i32(0)
            }
            other => Err(Error::custom(format!("expected i32, got {other:?}"))),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_i64(integer.as_i64()?)
                } else {
                    unreachable!("read_atom should never return anything else")
                }
            }
            Kind::Special if matches!(atom.nucleus, Some(Nucleus::Unit) | None) => {
                visitor.visit_i64(0)
            }
            other => Err(Error::custom(format!("expected i64, got {other:?}"))),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_i128(integer.as_i128()?)
                } else {
                    unreachable!("read_atom should never return anything else")
                }
            }
            Kind::Special if matches!(atom.nucleus, Some(Nucleus::Unit) | None) => {
                visitor.visit_i128(0)
            }
            other => Err(Error::custom(format!("expected i128, got {other:?}"))),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_u8(integer.as_u8()?)
                } else {
                    unreachable!("read_atom should never return anything else")
                }
            }
            Kind::Special if matches!(atom.nucleus, Some(Nucleus::Unit) | None) => {
                visitor.visit_u8(0)
            }
            other => Err(Error::custom(format!("expected u8, got {other:?}"))),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_u16(integer.as_u16()?)
                } else {
                    unreachable!("read_atom should never return anything else")
                }
            }
            Kind::Special if matches!(atom.nucleus, Some(Nucleus::Unit) | None) => {
                visitor.visit_u16(0)
            }
            other => Err(Error::custom(format!("expected u16, got {other:?}"))),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_u32(integer.as_u32()?)
                } else {
                    unreachable!("read_atom should never return anything else")
                }
            }
            Kind::Special if matches!(atom.nucleus, Some(Nucleus::Unit) | None) => {
                visitor.visit_u32(0)
            }
            other => Err(Error::custom(format!("expected u32, got {other:?}"))),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_u64(integer.as_u64()?)
                } else {
                    unreachable!("read_atom should never return anything else")
                }
            }
            Kind::Special if matches!(atom.nucleus, Some(Nucleus::Unit) | None) => {
                visitor.visit_u64(0)
            }
            other => Err(Error::custom(format!("expected u64, got {other:?}"))),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_u128(integer.as_u128()?)
                } else {
                    unreachable!("read_atom should never return anything else")
                }
            }
            Kind::Special if matches!(atom.nucleus, Some(Nucleus::Unit) | None) => {
                visitor.visit_u128(0)
            }
            other => Err(Error::custom(format!("expected i64, got {other:?}"))),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_f32(integer.as_f32()?)
                } else {
                    unreachable!("read_atom should never return anything else")
                }
            }

            Kind::Float => {
                if let Some(Nucleus::Float(float)) = atom.nucleus {
                    visitor.visit_f32(float.as_f32()?)
                } else {
                    unreachable!("read_atom should never return anything else")
                }
            }
            Kind::Special if matches!(atom.nucleus, Some(Nucleus::Unit) | None) => {
                visitor.visit_f32(0.)
            }
            other => Err(Error::custom(format!("expected f32, got {other:?}"))),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_f64(integer.as_f64()?)
                } else {
                    unreachable!("read_atom should never return anything else")
                }
            }

            Kind::Float => {
                if let Some(Nucleus::Float(float)) = atom.nucleus {
                    visitor.visit_f64(float.as_f64())
                } else {
                    unreachable!("read_atom should never return anything else")
                }
            }
            Kind::Special if matches!(atom.nucleus, Some(Nucleus::Unit) | None) => {
                visitor.visit_f64(0.)
            }
            other => Err(Error::custom(format!("expected f64, got {other:?}"))),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_char(
                        char::from_u32(integer.as_u32()?)
                            .ok_or_else(|| Error::InvalidUtf8(String::from("invalid char")))?,
                    )
                } else {
                    unreachable!("read_atom should never return anything else")
                }
            }
            Kind::Special if matches!(atom.nucleus, Some(Nucleus::Unit) | None) => {
                visitor.visit_char('\0')
            }
            other => Err(Error::custom(format!("expected char, got {other:?}"))),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::Bytes => match atom.nucleus {
                Some(Nucleus::Bytes(bytes)) => match bytes {
                    BufferedBytes::Data(bytes) => {
                        visitor.visit_borrowed_str(std::str::from_utf8(bytes)?)
                    }
                    BufferedBytes::Scratch => {
                        visitor.visit_str(std::str::from_utf8(&self.scratch)?)
                    }
                },
                _ => unreachable!("read_atom should never return anything else"),
            },
            Kind::Symbol => self.visit_symbol(&atom, visitor),
            Kind::Special => {
                if matches!(atom.nucleus, Some(Nucleus::Named)) {
                    // If we encounter a named entity here, skip it and trust that serde will decode the following information correctly.
                    self.deserialize_str(visitor)
                } else if matches!(atom.nucleus, Some(Nucleus::Unit) | None) {
                    visitor.visit_borrowed_str("")
                } else {
                    self.visit_symbol(&atom, visitor)
                }
            }
            other => Err(Error::custom(format!("expected str, got {other:?}"))),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::Bytes => match atom.nucleus {
                Some(Nucleus::Bytes(bytes)) => match bytes {
                    BufferedBytes::Data(bytes) => visitor.visit_borrowed_bytes(bytes),
                    BufferedBytes::Scratch => visitor.visit_bytes(&self.scratch),
                },
                _ => unreachable!("read_atom should never return anything else"),
            },
            Kind::Sequence => {
                let mut buffer = Vec::with_capacity(atom.arg as usize);
                for _ in 0..atom.arg {
                    let atom = self.read_atom()?;

                    if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                        buffer.push(integer.as_u8()?);
                    } else {
                        return Err(Error::custom(
                            "expected byte array, encountered non-integer atom",
                        ));
                    }
                }
                visitor.visit_byte_buf(buffer)
            }
            Kind::Special if matches!(atom.nucleus, Some(Nucleus::Unit) | None) => {
                visitor.visit_borrowed_bytes(b"")
            }
            other => Err(Error::custom(format!("expected bytes, got {other:?}"))),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.peek_atom()?;
        if matches!(atom.kind, Kind::Special) && atom.nucleus.is_none() {
            // Consume the atom.
            self.read_atom()?;
            return visitor.visit_none();
        }

        visitor.visit_some(self)
    }

    // In Serde, unit means an anonymous value containing no data.
    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        if atom.kind == Kind::Special && matches!(atom.nucleus, Some(Nucleus::Unit)) {
            visitor.visit_unit()
        } else {
            Err(Error::custom(format!("expected unit, got {:?}", atom.kind)))
        }
    }

    // Unit struct means a named value containing no data.
    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain. That means not
    // parsing anything other than the contained value.
    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        if atom.kind == Kind::Sequence {
            visitor.visit_seq(AtomList::new(self, Some(atom.arg as usize)))
        } else if atom.kind == Kind::Special && matches!(atom.nucleus, Some(Nucleus::Unit) | None) {
            visitor.visit_seq(EmptyList)
        } else {
            Err(Error::custom(format!(
                "expected sequence, got {:?}",
                atom.kind
            )))
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match (atom.kind, atom.nucleus) {
            (Kind::Map, _) => visitor.visit_map(AtomList::new(self, Some(atom.arg as usize))),
            (Kind::Special, Some(Nucleus::DynamicMap)) => {
                visitor.visit_map(AtomList::new(self, None))
            }
            (Kind::Special, Some(Nucleus::Unit) | None) => visitor.visit_map(EmptyList),
            (kind, _) => Err(Error::custom(format!("expected map, got {kind:?}"))),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::Symbol => self.visit_symbol(&atom, visitor),
            Kind::Bytes => {
                if let Some(Nucleus::Bytes(bytes)) = atom.nucleus {
                    let as_str = std::str::from_utf8(bytes.as_slice(&self.scratch))
                        .map_err(|err| Error::InvalidUtf8(err.to_string()))?;
                    visitor.visit_str(as_str)
                } else {
                    unreachable!("read_atom shouldn't return anything else")
                }
            }
            other => Err(Error::custom(format!("expected identifier, got {other:?}"))),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct EmptyList;

impl<'de> MapAccess<'de> for EmptyList {
    type Error = Error;

    #[inline]
    fn next_key_seed<K>(&mut self, _seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        Ok(None)
    }

    #[inline]
    fn next_value_seed<V>(&mut self, _seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        unreachable!()
    }
}

impl<'de> SeqAccess<'de> for EmptyList {
    type Error = Error;

    #[inline]
    fn next_element_seed<T>(&mut self, _seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        Ok(None)
    }
}

struct AtomList<'a, 's, 'de, R: Reader<'de>> {
    de: &'a mut Deserializer<'s, 'de, R>,
    consumed: usize,
    count: Option<usize>,
    eof: bool,
}

impl<'a, 's, 'de, R: Reader<'de>> AtomList<'a, 's, 'de, R> {
    fn new(de: &'a mut Deserializer<'s, 'de, R>, count: Option<usize>) -> Self {
        Self {
            de,
            count,
            consumed: 0,
            eof: false,
        }
    }

    fn check_is_eof(&mut self) -> Result<bool> {
        if self.eof {
            return Ok(true);
        } else if let Some(count) = self.count {
            if count == self.consumed {
                self.eof = true;
                return Ok(true);
            }
        } else {
            let atom = self.de.peek_atom()?;
            if matches!(atom.kind, Kind::Special)
                && matches!(atom.nucleus, Some(Nucleus::DynamicEnd))
            {
                self.eof = true;
                self.de.read_atom()?;
                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl<'a, 's, 'de, R: Reader<'de>> Debug for AtomList<'a, 's, 'de, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AtomList")
            .field("de", &self.de)
            .field("consumed", &self.consumed)
            .field("count", &self.count)
            .field("eof", &self.eof)
            .finish()
    }
}

impl<'a, 's, 'de, R: Reader<'de>> SeqAccess<'de> for AtomList<'a, 's, 'de, R> {
    type Error = Error;

    #[cfg_attr(feature = "tracing", instrument(skip(seed)))]
    #[inline]
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.check_is_eof()? {
            Ok(None)
        } else {
            self.consumed += 1;
            seed.deserialize(&mut *self.de).map(Some)
        }
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.count
    }
}

impl<'a, 's, 'de, R: Reader<'de>> MapAccess<'de> for AtomList<'a, 's, 'de, R> {
    type Error = Error;

    #[cfg_attr(feature = "tracing", instrument(skip(seed)))]
    #[inline]
    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.check_is_eof()? {
            Ok(None)
        } else {
            self.consumed += 1;
            seed.deserialize(&mut *self.de).map(Some)
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(seed)))]
    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // Deserialize a map value.
        seed.deserialize(&mut *self.de)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.count
    }
}

impl<'a, 's, 'de, R: Reader<'de>> EnumAccess<'de> for &'a mut Deserializer<'s, 'de, R> {
    type Error = Error;
    type Variant = Self;

    #[cfg_attr(feature = "tracing", instrument(skip(seed)))]
    #[inline]
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        // Have the seed deserialize the next atom, which should be the symbol.
        let atom = self.read_atom()?;
        if atom.kind == Kind::Special && matches!(atom.nucleus, Some(Nucleus::Named)) {
            let val = seed.deserialize(&mut *self)?;
            Ok((val, self))
        } else {
            Err(Error::custom(format!(
                "expected Named, got {:?}",
                atom.kind
            )))
        }
    }
}

impl<'a, 's, 'de, R: Reader<'de>> VariantAccess<'de> for &'a mut Deserializer<'s, 'de, R> {
    type Error = Error;

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument(skip(seed)))]
    #[inline]
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self, visitor)
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[inline]
    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self, visitor)
    }
}

/// A collection of deserialized symbols.
#[derive(Debug)]
pub enum SymbolMap<'a, 'de> {
    /// An owned list of symbols.
    Owned(SymbolList<'static>),
    /// A mutable reference to an owned list of symbols.
    Persistent(&'a mut SymbolList<'static>),
    /// A list of borrowed symbols.
    Borrowed(SymbolList<'de>),
}

impl<'de> SymbolMap<'static, 'de> {
    /// Returns a new symbol map that will persist symbols between payloads.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self::Owned(SymbolList::new())
    }

    /// Returns a deserializer for `slice`.
    #[inline]
    pub fn deserializer_for_slice<'a>(
        &'a mut self,
        slice: &'de [u8],
    ) -> Result<Deserializer<'a, 'de, SliceReader<'de>>> {
        Deserializer::from_slice_with_symbols(slice, self.persistent(), usize::MAX)
    }

    #[must_use]
    fn persistent(&mut self) -> SymbolMap<'_, 'de> {
        match self {
            Self::Owned(vec) => SymbolMap::Persistent(vec),
            Self::Persistent(vec) => SymbolMap::Persistent(vec),
            Self::Borrowed(_) => unreachable!(),
        }
    }
}

impl<'a, 'de> SymbolMap<'a, 'de> {
    #[allow(clippy::cast_possible_truncation)]
    fn visit_symbol_id<V>(&self, symbol_id: u64, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Owned(vec) => {
                let symbol = vec
                    .get(symbol_id as usize)
                    .ok_or(Error::UnknownSymbol(symbol_id))?;
                visitor.visit_str(&symbol)
            }
            Self::Persistent(vec) => {
                let symbol = vec
                    .get(symbol_id as usize)
                    .ok_or(Error::UnknownSymbol(symbol_id))?;
                visitor.visit_str(&symbol)
            }
            Self::Borrowed(vec) => {
                let symbol = vec
                    .get(symbol_id as usize)
                    .ok_or(Error::UnknownSymbol(symbol_id))?;
                match symbol {
                    SymbolStr::Data(symbol) => visitor.visit_borrowed_str(symbol),
                    SymbolStr::InList(symbol) => visitor.visit_str(symbol),
                }
            }
        }
    }

    fn push(&mut self, symbol: &str) {
        #[allow(clippy::match_same_arms)] // false positive due to lifetimes
        match self {
            Self::Owned(vec) => vec.push(symbol),
            Self::Persistent(vec) => vec.push(symbol),
            Self::Borrowed(vec) => vec.push(symbol),
        }
    }

    fn push_borrowed(&mut self, symbol: &'de str) {
        match self {
            Self::Owned(vec) => vec.push(symbol),
            Self::Persistent(vec) => vec.push(symbol),
            Self::Borrowed(vec) => vec.push_borrowed(symbol),
        }
    }
}

/// A collection of symbols accumulated during deserialization.
#[derive(Debug)]
pub struct SymbolList<'de> {
    buffer: String,
    entries: Vec<SymbolListEntry<'de>>,
}

impl<'de> SymbolList<'de> {
    const fn new() -> Self {
        Self {
            buffer: String::new(),
            entries: Vec::new(),
        }
    }

    /// Push a symbol that has been borrowed from the deserialization source.
    #[inline]
    pub fn push_borrowed(&mut self, borrowed: &'de str) {
        self.entries.push(SymbolListEntry::Borrowed(borrowed));
    }

    /// Push a symbol that cannot be borrowed from the deserialization source.
    #[inline]
    pub fn push(&mut self, ephemeral: &str) {
        let start = self.buffer.len();
        self.buffer.push_str(ephemeral);
        self.entries
            .push(SymbolListEntry::Buffer(start..self.buffer.len()));
    }

    /// Return the symbol stored at `index`, or `None` if `index` is out of
    /// bounds.
    #[inline]
    #[must_use]
    pub fn get(&self, index: usize) -> Option<SymbolStr<'de, '_>> {
        match self.entries.get(index)? {
            SymbolListEntry::Buffer(range) => Some(SymbolStr::InList(&self.buffer[range.clone()])),
            SymbolListEntry::Borrowed(str) => Some(SymbolStr::Data(str)),
        }
    }
}

/// A symbol stored in a [`SymbolList`].
pub enum SymbolStr<'de, 'ephemeral> {
    /// A symbol that has been borrowed from the data being deserialized.
    Data(&'de str),
    /// A symbol that is stored inside of the [`SymbolList`].
    InList(&'ephemeral str),
}

impl Deref for SymbolStr<'_, '_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            SymbolStr::Data(str) | SymbolStr::InList(str) => str,
        }
    }
}

#[derive(Debug, Clone)]
enum SymbolListEntry<'de> {
    Buffer(Range<usize>),
    Borrowed(&'de str),
}
