use std::marker::PhantomData;

use byteorder::LittleEndian;
use format::Kind;
use serde::de::{self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
#[cfg(feature = "tracing")]
use tracing::instrument;

use crate::{
    format::{self, Atom, Nucleus, CURRENT_VERSION},
    reader::{Reader, SliceReader},
    Error, Result,
};

#[derive(Debug)]
pub struct Deserializer<'s, 'de, R: Reader<'de>> {
    input: R,
    symbols: SymbolMap<'s, 'de>,
    peeked_atom: Option<Atom<'de>>,
    _phantom: PhantomData<&'de R>,
}

impl<'s, 'de> Deserializer<'s, 'de, SliceReader<'de>> {
    pub fn from_slice(input: &'de [u8]) -> Result<Self> {
        Self::from_slice_with_symbols(input, SymbolMap::new())
    }

    fn from_slice_with_symbols(input: &'de [u8], mut symbols: SymbolMap<'s, 'de>) -> Result<Self> {
        // TODO make this configurable
        symbols.reserve(1024);
        let mut deserializer = Deserializer {
            input: SliceReader::from(input),
            symbols,
            peeked_atom: None,
            _phantom: PhantomData::default(),
        };
        deserializer.read_header()?;
        Ok(deserializer)
    }

    #[must_use]
    pub const fn end_of_input(&self) -> bool {
        self.input.data.is_empty() && self.peeked_atom.is_none()
    }
}

impl<'s, 'de, R: Reader<'de>> Deserializer<'s, 'de, R> {
    fn read_header(&mut self) -> Result<()> {
        let version = format::read_header(&mut self.input)?;
        if version == CURRENT_VERSION {
            Ok(())
        } else {
            Err(Error::InvalidData)
        }
    }

    fn read_atom(&mut self) -> Result<Atom<'de>> {
        if let Some(peeked) = self.peeked_atom.take() {
            Ok(peeked)
        } else {
            format::read_atom(&mut self.input)
        }
    }

    #[allow(clippy::missing_panics_doc)]
    fn peek_atom(&mut self) -> Result<&Atom<'_>> {
        if self.peeked_atom.is_none() {
            self.peeked_atom = Some(self.read_atom()?);
        }

        Ok(self.peeked_atom.as_ref().unwrap())
    }

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
            let name = self.input.buffered_read_bytes(arg as usize)?;
            let name = std::str::from_utf8(name)?;
            self.symbols.push(name);
            visitor.visit_borrowed_str(name)
        }
    }
}

impl<'a, 'de, 's, R: Reader<'de>> de::Deserializer<'de> for &'a mut Deserializer<'s, 'de, R> {
    type Error = Error;

    fn is_human_readable(&self) -> bool {
        false
    }

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[allow(clippy::cast_possible_truncation)]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;

        match atom.kind {
            Kind::None => visitor.visit_none(),
            Kind::Int => match atom.arg + 1 {
                1 => visitor.visit_i8(self.input.read_i8()?),
                2 => visitor.visit_i16(self.input.read_i16::<LittleEndian>()?),
                3 => visitor.visit_i32(self.input.read_i24::<LittleEndian>()?),
                4 => visitor.visit_i32(self.input.read_i32::<LittleEndian>()?),
                6 => visitor.visit_i64(self.input.read_i48::<LittleEndian>()?),
                8 => visitor.visit_i64(self.input.read_i64::<LittleEndian>()?),
                _ => Err(Error::InvalidData),
            },
            Kind::UInt => match atom.arg + 1 {
                1 => visitor.visit_u8(self.input.read_u8()?),
                2 => visitor.visit_u16(self.input.read_u16::<LittleEndian>()?),
                3 => visitor.visit_u32(self.input.read_u24::<LittleEndian>()?),
                4 => visitor.visit_u32(self.input.read_u32::<LittleEndian>()?),
                6 => visitor.visit_u64(self.input.read_u48::<LittleEndian>()?),
                8 => visitor.visit_u64(self.input.read_u64::<LittleEndian>()?),
                _ => Err(Error::InvalidData),
            },
            Kind::Float => match atom.arg + 1 {
                4 => visitor.visit_f32(self.input.read_f32::<LittleEndian>()?),
                8 => visitor.visit_f64(self.input.read_f64::<LittleEndian>()?),
                _ => Err(Error::InvalidData),
            },
            Kind::Sequence => visitor.visit_seq(AtomList::new(self, atom.arg as usize)),
            Kind::Map => visitor.visit_map(AtomList::new(self, atom.arg as usize)),
            Kind::Symbol => self.visit_symbol(&atom, visitor),
            Kind::Bytes => match &atom.nucleus {
                Some(Nucleus::Bytes(bytes)) => visitor.visit_borrowed_bytes(bytes),
                None => visitor.visit_none(),
                _ => Err(Error::InvalidData),
            },
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::None => visitor.visit_bool(false),
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_bool(integer.as_i8()? != 0)
                } else {
                    Err(Error::InvalidData)
                }
            }
            _ => Err(Error::InvalidData),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::None => visitor.visit_i8(0),
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_i8(integer.as_i8()?)
                } else {
                    Err(Error::InvalidData)
                }
            }
            _ => Err(Error::InvalidData),
        }
    }
    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::None => visitor.visit_i16(0),
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_i16(integer.as_i16()?)
                } else {
                    Err(Error::InvalidData)
                }
            }
            _ => Err(Error::InvalidData),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::None => visitor.visit_i32(0),
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_i32(integer.as_i32()?)
                } else {
                    Err(Error::InvalidData)
                }
            }
            _ => Err(Error::InvalidData),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::None => visitor.visit_i64(0),
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_i64(integer.as_i64()?)
                } else {
                    Err(Error::InvalidData)
                }
            }
            _ => Err(Error::InvalidData),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::None => visitor.visit_u8(0),
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_u8(integer.as_u8()?)
                } else {
                    Err(Error::InvalidData)
                }
            }
            _ => Err(Error::InvalidData),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::None => visitor.visit_u16(0),
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_u16(integer.as_u16()?)
                } else {
                    Err(Error::InvalidData)
                }
            }
            _ => Err(Error::InvalidData),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::None => visitor.visit_u32(0),
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_u32(integer.as_u32()?)
                } else {
                    Err(Error::InvalidData)
                }
            }
            _ => Err(Error::InvalidData),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::None => visitor.visit_u64(0),
            Kind::UInt | Kind::Int => {
                if let Some(Nucleus::Integer(integer)) = atom.nucleus {
                    visitor.visit_u64(integer.as_u64()?)
                } else {
                    Err(Error::InvalidData)
                }
            }
            _ => Err(Error::InvalidData),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
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
                    Err(Error::InvalidData)
                }
            }

            Kind::Float => {
                if let Some(Nucleus::Float(float)) = atom.nucleus {
                    visitor.visit_f32(float.as_f32()?)
                } else {
                    Err(Error::InvalidData)
                }
            }
            _ => Err(Error::InvalidData),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
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
                    Err(Error::InvalidData)
                }
            }

            Kind::Float => {
                if let Some(Nucleus::Float(float)) = atom.nucleus {
                    visitor.visit_f64(float.as_f64())
                } else {
                    Err(Error::InvalidData)
                }
            }
            _ => Err(Error::InvalidData),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(_visitor)))]
    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::Bytes => match atom.nucleus {
                Some(Nucleus::Bytes(bytes)) => {
                    visitor.visit_borrowed_str(std::str::from_utf8(bytes)?)
                }
                _ => Err(Error::InvalidData),
            },
            _ => Err(Error::InvalidData),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    #[cfg_attr(feature = "tracing", instrument(skip(_visitor)))]
    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.peek_atom()?;
        match atom.kind {
            Kind::None => {
                // Consume the atom.
                drop(self.read_atom()?);
                visitor.visit_none()
            }
            _ => visitor.visit_some(self),
        }
    }

    // In Serde, unit means an anonymous value containing no data.
    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        if atom.kind == Kind::None {
            visitor.visit_unit()
        } else {
            Err(Error::InvalidData)
        }
    }

    // Unit struct means a named value containing no data.
    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
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
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    #[allow(clippy::cast_possible_truncation)]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        if atom.kind == Kind::Sequence {
            visitor.visit_seq(AtomList::new(self, atom.arg as usize))
        } else {
            Err(Error::InvalidData)
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

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
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        if atom.kind == Kind::Map {
            visitor.visit_map(AtomList::new(self, atom.arg as usize))
        } else {
            Err(Error::InvalidData)
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
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
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let atom = self.read_atom()?;
        match atom.kind {
            Kind::Symbol => self.visit_symbol(&atom, visitor),
            _ => Err(Error::InvalidData),
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

#[derive(Debug)]
struct AtomList<'a, 's, 'de, R: Reader<'de>> {
    de: &'a mut Deserializer<'s, 'de, R>,
    consumed: usize,
    count: usize,
}

impl<'a, 's, 'de, R: Reader<'de>> AtomList<'a, 's, 'de, R> {
    fn new(de: &'a mut Deserializer<'s, 'de, R>, count: usize) -> Self {
        Self {
            de,
            count,
            consumed: 0,
        }
    }
}

impl<'a, 's, 'de, R: Reader<'de>> SeqAccess<'de> for AtomList<'a, 's, 'de, R> {
    type Error = Error;

    #[cfg_attr(feature = "tracing", instrument(skip(seed)))]
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.count == self.consumed {
            Ok(None)
        } else {
            self.consumed += 1;
            seed.deserialize(&mut *self.de).map(Some)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.count)
    }
}

impl<'a, 's, 'de, R: Reader<'de>> MapAccess<'de> for AtomList<'a, 's, 'de, R> {
    type Error = Error;

    #[cfg_attr(feature = "tracing", instrument(skip(seed)))]
    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.count == self.consumed {
            Ok(None)
        } else {
            self.consumed += 1;
            seed.deserialize(&mut *self.de).map(Some)
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(seed)))]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // Deserialize a map value.
        seed.deserialize(&mut *self.de)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.count)
    }
}

impl<'a, 's, 'de, R: Reader<'de>> EnumAccess<'de> for &'a mut Deserializer<'s, 'de, R> {
    type Error = Error;
    type Variant = Self;

    #[cfg_attr(feature = "tracing", instrument(skip(seed)))]
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        // Have the seed deserialize the next atom, which should be the symbol
        let val = seed.deserialize(&mut *self)?;
        Ok((val, self))
    }
}

impl<'a, 's, 'de, R: Reader<'de>> VariantAccess<'de> for &'a mut Deserializer<'s, 'de, R> {
    type Error = Error;

    #[cfg_attr(feature = "tracing", instrument)]
    fn unit_variant(self) -> Result<()> {
        if self.read_atom()?.kind == Kind::None {
            Ok(())
        } else {
            Err(Error::InvalidData)
        }
    }

    #[cfg_attr(feature = "tracing", instrument(skip(seed)))]
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self, visitor)
    }

    #[cfg_attr(feature = "tracing", instrument(skip(visitor)))]
    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self, visitor)
    }
}

#[derive(Debug)]
pub enum SymbolMap<'a, 'de> {
    Owned(Vec<String>),
    Persistent(&'a mut Vec<String>),
    Borrowed(Vec<&'de str>),
}

impl<'de> SymbolMap<'static, 'de> {
    #[must_use]
    pub const fn new() -> Self {
        Self::Owned(Vec::new())
    }

    pub fn deserializer_for_slice<'a>(
        &'a mut self,
        slice: &'de [u8],
    ) -> Result<Deserializer<'a, 'de, SliceReader<'de>>> {
        Deserializer::from_slice_with_symbols(slice, self.persistent())
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
                let symbol = vec.get(symbol_id as usize).ok_or(Error::InvalidData)?;
                visitor.visit_str(symbol)
            }
            Self::Persistent(vec) => {
                let symbol = vec.get(symbol_id as usize).ok_or(Error::InvalidData)?;
                visitor.visit_str(symbol)
            }
            Self::Borrowed(vec) => {
                let symbol = vec.get(symbol_id as usize).ok_or(Error::InvalidData)?;
                visitor.visit_borrowed_str(*symbol)
            }
        }
    }

    fn reserve(&mut self, amount: usize) {
        match self {
            Self::Owned(vec) => vec.reserve(amount),
            Self::Persistent(vec) => vec.reserve(amount),
            Self::Borrowed(vec) => vec.reserve(amount),
        }
    }

    fn push(&mut self, symbol: &'de str) {
        match self {
            Self::Owned(vec) => vec.push(symbol.to_string()),
            Self::Persistent(vec) => vec.push(symbol.to_string()),
            Self::Borrowed(vec) => vec.push(symbol),
        }
    }
}
