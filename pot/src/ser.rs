use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    usize,
};

use byteorder::WriteBytesExt;
use serde::{ser, Serialize};
#[cfg(feature = "tracing")]
use tracing::instrument;

use crate::{
    format::{self, Kind, CURRENT_VERSION},
    Error, Result,
};

/// A `Pot` serializer.
#[derive(Debug)]
pub struct Serializer<'a, W: WriteBytesExt> {
    symbol_map: SymbolMapRef<'a>,
    output: W,
    bytes_written: usize,
}

impl<'a, W: WriteBytesExt + Debug> Serializer<'a, W> {
    /// Returns a new serializer outputting written bytes into `output`.
    pub fn new(output: W) -> Result<Self> {
        Self::new_with_symbol_map(output, SymbolMapRef::Owned(SymbolMap::default()))
    }

    fn new_with_symbol_map(mut output: W, symbol_map: SymbolMapRef<'a>) -> Result<Self> {
        let bytes_written = format::write_header(&mut output, CURRENT_VERSION)?;
        Ok(Self {
            symbol_map,
            output,
            bytes_written,
        })
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn write_symbol(&mut self, symbol: &'static str) -> Result<()> {
        let registered_symbol = self.symbol_map.find_or_add(symbol);
        if registered_symbol.new {
            // The arg is the length followed by a 0 bit.
            let arg = (symbol.len() as u64) << 1;
            self.bytes_written +=
                format::write_atom_header(&mut self.output, Kind::Symbol, Some(arg))?;
            self.output.write_all(symbol.as_bytes())?;
            self.bytes_written += symbol.len() as usize;
            Ok(())
        } else {
            // When a symbol was already emitted, just emit the id followed by a 1 bit.
            self.bytes_written += format::write_atom_header(
                &mut self.output,
                Kind::Symbol,
                Some(u64::from((registered_symbol.id << 1) | 1)),
            )?;
            Ok(())
        }
    }
}

impl<'de, 'a: 'de, W: WriteBytesExt + Debug> ser::Serializer for &'de mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn is_human_readable(&self) -> bool {
        false
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.bytes_written += format::write_u8(&mut self.output, v as u8)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_i8(self, v: i8) -> Result<()> {
        self.bytes_written += format::write_i8(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_i16(self, v: i16) -> Result<()> {
        self.bytes_written += format::write_i16(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_i32(self, v: i32) -> Result<()> {
        self.bytes_written += format::write_i32(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_i64(self, v: i64) -> Result<()> {
        self.bytes_written += format::write_i64(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_i128(self, v: i128) -> Result<()> {
        self.bytes_written += format::write_i128(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_u8(self, v: u8) -> Result<()> {
        self.bytes_written += format::write_u8(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_u16(self, v: u16) -> Result<()> {
        self.bytes_written += format::write_u16(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_u32(self, v: u32) -> Result<()> {
        self.bytes_written += format::write_u32(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_u64(self, v: u64) -> Result<()> {
        self.bytes_written += format::write_u64(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_u128(self, v: u128) -> Result<()> {
        self.bytes_written += format::write_u128(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_f32(self, v: f32) -> Result<()> {
        self.bytes_written += format::write_f32(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_f64(self, v: f64) -> Result<()> {
        self.bytes_written += format::write_f64(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_char(self, v: char) -> Result<()> {
        self.bytes_written += format::write_u32(&mut self.output, v as u32)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_str(self, v: &str) -> Result<()> {
        self.bytes_written += format::write_str(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.bytes_written += format::write_bytes(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_none(self) -> Result<()> {
        self.bytes_written += format::write_none(&mut self.output)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument(skip(value)))]
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_unit(self) -> Result<()> {
        self.bytes_written += format::write_unit(&mut self.output)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.write_symbol(variant)?;
        self.bytes_written += format::write_none(&mut self.output)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument(skip(value)))]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[cfg_attr(feature = "tracing", instrument(skip(value)))]
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.write_symbol(variant)?;
        value.serialize(&mut *self)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let len = len.ok_or(Error::SequenceSizeMustBeKnown)?;
        self.bytes_written +=
            format::write_atom_header(&mut self.output, Kind::Sequence, Some(len as u64))?;
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.write_symbol(variant)?;
        self.serialize_seq(Some(len))
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        let len = len.ok_or(Error::SequenceSizeMustBeKnown)?;
        self.bytes_written +=
            format::write_atom_header(&mut self.output, Kind::Map, Some(len as u64))?;
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    #[cfg_attr(feature = "tracing", instrument)]
    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.write_symbol(variant)?;
        self.serialize_struct(name, len)
    }
}

impl<'de, 'a: 'de, W: WriteBytesExt + Debug> ser::SerializeSeq for &'de mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'de, 'a: 'de, W: WriteBytesExt + Debug> ser::SerializeTuple for &'de mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'de, 'a: 'de, W: WriteBytesExt + Debug> ser::SerializeTupleStruct
    for &'de mut Serializer<'a, W>
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'de, 'a: 'de, W: WriteBytesExt + Debug> ser::SerializeTupleVariant
    for &'de mut Serializer<'a, W>
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'de, 'a: 'de, W: WriteBytesExt + Debug> ser::SerializeMap for &'de mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'de, 'a: 'de, W: WriteBytesExt + Debug> ser::SerializeStruct for &'de mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.write_symbol(key)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'de, 'a: 'de, W: WriteBytesExt + Debug> ser::SerializeStructVariant
    for &'de mut Serializer<'a, W>
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.write_symbol(key)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

/// A list of previously serialized symbols.
#[derive(Debug)]
pub struct SymbolMap {
    symbols: Vec<(usize, u32)>,
}

impl Default for SymbolMap {
    fn default() -> Self {
        let mut symbols = Vec::default();
        // TODO make this configurable
        symbols.reserve(1024);
        Self { symbols }
    }
}

struct RegisteredSymbol {
    id: u32,
    new: bool,
}

impl SymbolMap {
    /// Returns a serializer that writes into `output` that persists symbols
    /// into `self`.
    pub fn serializer_for<W: WriteBytesExt + Debug>(
        &mut self,
        output: W,
    ) -> Result<Serializer<'_, W>> {
        Serializer::new_with_symbol_map(output, SymbolMapRef::Borrowed(self))
    }

    #[allow(clippy::cast_possible_truncation)]
    fn find_or_add(&mut self, symbol: &'static str) -> RegisteredSymbol {
        // Symbols have to be static strings, and so we can rely on the addres
        // not changing. To avoid string comparisons, we're going to use the
        // address of the str in the map.
        let symbol_address = symbol.as_ptr() as usize;
        // Perform a binary search to find this existing element.
        match self
            .symbols
            .binary_search_by(|check| symbol_address.cmp(&check.0))
        {
            Ok(position) => RegisteredSymbol {
                id: self.symbols[position].1,
                new: false,
            },
            Err(position) => {
                let id = self.symbols.len() as u32;
                self.symbols.insert(position, (symbol_address, id));
                RegisteredSymbol { id, new: true }
            }
        }
    }
}

#[derive(Debug)]
enum SymbolMapRef<'a> {
    Owned(SymbolMap),
    Borrowed(&'a mut SymbolMap),
}

impl<'a> Deref for SymbolMapRef<'a> {
    type Target = SymbolMap;

    fn deref(&self) -> &Self::Target {
        match self {
            SymbolMapRef::Owned(map) => map,
            SymbolMapRef::Borrowed(map) => map,
        }
    }
}

impl<'a> DerefMut for SymbolMapRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            SymbolMapRef::Owned(map) => map,
            SymbolMapRef::Borrowed(map) => map,
        }
    }
}
