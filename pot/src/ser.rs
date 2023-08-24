use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use std::ops::Range;
use std::usize;

use byteorder::WriteBytesExt;
use serde::de::{SeqAccess, Visitor};
use serde::{ser, Deserialize, Serialize};
#[cfg(feature = "tracing")]
use tracing::instrument;

use crate::format::{self, Kind, Special, CURRENT_VERSION};
use crate::{Error, Result};

/// A Pot serializer.
pub struct Serializer<'a, W: WriteBytesExt> {
    symbol_map: SymbolMapRef<'a>,
    output: W,
    bytes_written: usize,
}

impl<'a, W: WriteBytesExt> Debug for Serializer<'a, W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Serializer")
            .field("symbol_map", &self.symbol_map)
            .field("bytes_written", &self.bytes_written)
            .finish()
    }
}

impl<'a, W: WriteBytesExt> Serializer<'a, W> {
    /// Returns a new serializer outputting written bytes into `output`.
    #[inline]
    pub fn new(output: W) -> Result<Self> {
        Self::new_with_symbol_map(
            output,
            SymbolMapRef::Ephemeral(EphemeralSymbolMap::default()),
        )
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
            self.bytes_written += format::write_atom_header(&mut self.output, Kind::Symbol, arg)?;
            self.output.write_all(symbol.as_bytes())?;
            self.bytes_written += symbol.len();
        } else {
            // When a symbol was already emitted, just emit the id followed by a 1 bit.
            self.bytes_written += format::write_atom_header(
                &mut self.output,
                Kind::Symbol,
                u64::from((registered_symbol.id << 1) | 1),
            )?;
        }
        Ok(())
    }
}

impl<'de, 'a: 'de, W: WriteBytesExt + 'a> ser::Serializer for &'de mut Serializer<'a, W> {
    type Error = Error;
    type Ok = ();
    type SerializeMap = MapSerializer<'de, 'a, W>;
    type SerializeSeq = Self;
    type SerializeStruct = MapSerializer<'de, 'a, W>;
    type SerializeStructVariant = MapSerializer<'de, 'a, W>;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.bytes_written += format::write_bool(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_i8(self, v: i8) -> Result<()> {
        self.bytes_written += format::write_i8(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_i16(self, v: i16) -> Result<()> {
        self.bytes_written += format::write_i16(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_i32(self, v: i32) -> Result<()> {
        self.bytes_written += format::write_i32(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_i64(self, v: i64) -> Result<()> {
        self.bytes_written += format::write_i64(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_i128(self, v: i128) -> Result<()> {
        self.bytes_written += format::write_i128(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_u8(self, v: u8) -> Result<()> {
        self.bytes_written += format::write_u8(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_u16(self, v: u16) -> Result<()> {
        self.bytes_written += format::write_u16(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_u32(self, v: u32) -> Result<()> {
        self.bytes_written += format::write_u32(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_u64(self, v: u64) -> Result<()> {
        self.bytes_written += format::write_u64(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_u128(self, v: u128) -> Result<()> {
        self.bytes_written += format::write_u128(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_f32(self, v: f32) -> Result<()> {
        self.bytes_written += format::write_f32(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_f64(self, v: f64) -> Result<()> {
        self.bytes_written += format::write_f64(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_char(self, v: char) -> Result<()> {
        self.bytes_written += format::write_u32(&mut self.output, v as u32)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_str(self, v: &str) -> Result<()> {
        self.bytes_written += format::write_str(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.bytes_written += format::write_bytes(&mut self.output, v)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_none(self) -> Result<()> {
        self.bytes_written += format::write_none(&mut self.output)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(value)))]
    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_unit(self) -> Result<()> {
        self.bytes_written += format::write_unit(&mut self.output)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        format::write_named(&mut self.output)?;
        self.write_symbol(variant)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(value)))]
    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip(value)))]
    #[inline]
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
        format::write_named(&mut self.output)?;
        self.write_symbol(variant)?;
        value.serialize(&mut *self)?;
        Ok(())
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let len = len.ok_or(Error::SequenceSizeMustBeKnown)?;
        self.bytes_written +=
            format::write_atom_header(&mut self.output, Kind::Sequence, len as u64)?;
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        format::write_named(&mut self.output)?;
        self.write_symbol(variant)?;
        self.serialize_seq(Some(len))
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        if let Some(len) = len {
            self.bytes_written +=
                format::write_atom_header(&mut self.output, Kind::Map, len as u64)?;
            Ok(MapSerializer {
                serializer: self,
                known_length: true,
            })
        } else {
            self.bytes_written += format::write_special(&mut self.output, Special::DynamicMap)?;
            Ok(MapSerializer {
                serializer: self,
                known_length: false,
            })
        }
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    #[cfg_attr(feature = "tracing", instrument)]
    #[inline]
    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        format::write_named(&mut self.output)?;
        self.write_symbol(variant)?;
        self.serialize_struct(name, len)
    }
}

impl<'de, 'a: 'de, W: WriteBytesExt + 'a> ser::SerializeSeq for &'de mut Serializer<'a, W> {
    type Error = Error;
    type Ok = ();

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'de, 'a: 'de, W: WriteBytesExt + 'a> ser::SerializeTuple for &'de mut Serializer<'a, W> {
    type Error = Error;
    type Ok = ();

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'de, 'a: 'de, W: WriteBytesExt + 'a> ser::SerializeTupleStruct for &'de mut Serializer<'a, W> {
    type Error = Error;
    type Ok = ();

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'de, 'a: 'de, W: WriteBytesExt + 'a> ser::SerializeTupleVariant
    for &'de mut Serializer<'a, W>
{
    type Error = Error;
    type Ok = ();

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

/// Serializes map-like values.
pub struct MapSerializer<'de, 'a, W: WriteBytesExt> {
    serializer: &'de mut Serializer<'a, W>,
    known_length: bool,
}

impl<'de, 'a: 'de, W: WriteBytesExt + 'a> ser::SerializeMap for MapSerializer<'de, 'a, W> {
    type Error = Error;
    type Ok = ();

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut *self.serializer)
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn end(self) -> Result<()> {
        if !self.known_length {
            format::write_special(&mut self.serializer.output, Special::DynamicEnd)?;
        }
        Ok(())
    }
}

impl<'de, 'a: 'de, W: WriteBytesExt + 'a> ser::SerializeStruct for MapSerializer<'de, 'a, W> {
    type Error = Error;
    type Ok = ();

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serializer.write_symbol(key)?;
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn end(self) -> Result<()> {
        if !self.known_length {
            format::write_special(&mut self.serializer.output, Special::DynamicEnd)?;
        }
        Ok(())
    }
}

impl<'de, 'a: 'de, W: WriteBytesExt + 'a> ser::SerializeStructVariant
    for MapSerializer<'de, 'a, W>
{
    type Error = Error;
    type Ok = ();

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serializer.write_symbol(key)?;
        value.serialize(&mut *self.serializer)
    }

    #[inline]
    fn end(self) -> Result<()> {
        if !self.known_length {
            format::write_special(&mut self.serializer.output, Special::DynamicEnd)?;
        }
        Ok(())
    }
}

#[derive(Default)]
struct EphemeralSymbolMap {
    symbols: Vec<(&'static str, u32)>,
}

struct RegisteredSymbol {
    id: u32,
    new: bool,
}

impl EphemeralSymbolMap {
    #[allow(clippy::cast_possible_truncation)]
    fn find_or_add(&mut self, symbol: &'static str) -> RegisteredSymbol {
        // Symbols have to be static strings, and so we can rely on the addres
        // not changing. To avoid string comparisons, we're going to use the
        // address of the str in the map.
        let symbol_address = symbol.as_ptr() as usize;
        // Perform a binary search to find this existing element.
        match self
            .symbols
            .binary_search_by(|check| (check.0.as_ptr() as usize).cmp(&symbol_address))
        {
            Ok(position) => RegisteredSymbol {
                id: self.symbols[position].1,
                new: false,
            },
            Err(position) => {
                let id = self.symbols.len() as u32;
                self.symbols.insert(position, (symbol, id));
                RegisteredSymbol { id, new: true }
            }
        }
    }
}

impl Debug for EphemeralSymbolMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut set = f.debug_set();
        for index in SymbolIdSorter::new(&self.symbols, |sym| sym.1) {
            set.entry(&self.symbols[index].0);
        }
        set.finish()
    }
}

/// A list of previously serialized symbols.
pub struct SymbolMap {
    symbols: String,
    entries: Vec<(Range<usize>, u32)>,
    static_lookup: Vec<(usize, u32)>,
}

impl Debug for SymbolMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_set();
        for entry in &self.entries {
            s.entry(&&self.symbols[entry.0.clone()]);
        }
        s.finish()
    }
}

impl Default for SymbolMap {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolMap {
    /// Returns a new, empty symbol map.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            symbols: String::new(),
            entries: Vec::new(),
            static_lookup: Vec::new(),
        }
    }

    /// Returns a serializer that writes into `output` and persists symbols
    /// into `self`.
    #[inline]
    pub fn serializer_for<W: WriteBytesExt>(&mut self, output: W) -> Result<Serializer<'_, W>> {
        Serializer::new_with_symbol_map(output, SymbolMapRef::Persistent(self))
    }

    fn find_or_add(&mut self, symbol: &'static str) -> RegisteredSymbol {
        // Symbols have to be static strings, and so we can rely on the addres
        // not changing. To avoid string comparisons, we're going to use the
        // address of the str in the map.
        let symbol_address = symbol.as_ptr() as usize;
        // Perform a binary search to find this existing element.
        match self
            .static_lookup
            .binary_search_by(|check| symbol_address.cmp(&check.0))
        {
            Ok(position) => RegisteredSymbol {
                id: self.static_lookup[position].1,
                new: false,
            },
            Err(position) => {
                // This static symbol hasn't been encountered before.
                let symbol = self.find_entry_by_str(symbol);
                self.static_lookup
                    .insert(position, (symbol_address, symbol.id));
                symbol
            }
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn find_entry_by_str(&mut self, symbol: &str) -> RegisteredSymbol {
        match self
            .entries
            .binary_search_by(|check| self.symbols[check.0.clone()].cmp(symbol))
        {
            Ok(index) => RegisteredSymbol {
                id: self.entries[index].1,
                new: false,
            },
            Err(insert_at) => {
                let id = self.entries.len() as u32;
                let start = self.symbols.len();
                self.symbols.push_str(symbol);
                self.entries
                    .insert(insert_at, (start..self.symbols.len(), id));
                RegisteredSymbol { id, new: true }
            }
        }
    }

    /// Inserts `symbol` into this map.
    ///
    /// Returns true if this symbol had not previously been registered. Returns
    /// false if the symbol was already included in the map.
    pub fn insert(&mut self, symbol: &str) -> bool {
        self.find_entry_by_str(symbol).new
    }

    /// Returns the number of entries in this map.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the map has no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Adds all symbols encountered in `value`.
    ///
    /// Returns the number of symbols added.
    ///
    /// Due to how serde works, this function can only encounter symbols that
    /// are being used. For example, if `T` is an enum, only variant being
    /// passed in will have its name, and additional calls for each variant will
    /// be needed to ensure every symbol is added.
    pub fn populate_from<T>(&mut self, value: &T) -> Result<usize, SymbolMapPopulationError>
    where
        T: Serialize,
    {
        let start_count = self.entries.len();
        value.serialize(&mut SymbolMapPopulator(self))?;
        Ok(self.entries.len() - start_count)
    }
}

impl Serialize for SymbolMap {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for index in SymbolIdSorter::new(&self.entries, |entry| entry.1) {
            seq.serialize_element(&self.symbols[self.entries[index].0.clone()])?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for SymbolMap {
    #[inline]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(SymbolMapVisitor)
    }
}

struct SymbolMapVisitor;

impl<'de> Visitor<'de> for SymbolMapVisitor {
    type Value = SymbolMap;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("symbol map")
    }

    #[inline]
    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut map = SymbolMap::new();
        if let Some(hint) = seq.size_hint() {
            map.entries.reserve(hint);
        }
        let mut id = 0;
        while let Some(element) = seq.next_element::<Cow<'_, str>>()? {
            let start = map.symbols.len();
            map.symbols.push_str(&element);
            map.entries.push((start..map.symbols.len(), id));
            id += 1;
        }

        map.entries
            .sort_by(|a, b| map.symbols[a.0.clone()].cmp(&map.symbols[b.0.clone()]));

        Ok(map)
    }
}

#[derive(Debug)]
enum SymbolMapRef<'a> {
    Ephemeral(EphemeralSymbolMap),
    Persistent(&'a mut SymbolMap),
}

impl SymbolMapRef<'_> {
    fn find_or_add(&mut self, symbol: &'static str) -> RegisteredSymbol {
        match self {
            SymbolMapRef::Ephemeral(map) => map.find_or_add(symbol),
            SymbolMapRef::Persistent(map) => map.find_or_add(symbol),
        }
    }
}

struct SymbolMapPopulator<'a>(&'a mut SymbolMap);

impl<'ser, 'a> serde::ser::Serializer for &'ser mut SymbolMapPopulator<'a> {
    type Error = SymbolMapPopulationError;
    type Ok = ();
    type SerializeMap = Self;
    type SerializeSeq = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;

    #[inline]
    fn serialize_bool(self, _v: bool) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_i8(self, _v: i8) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_i16(self, _v: i16) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_i32(self, _v: i32) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_i64(self, _v: i64) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_u8(self, _v: u8) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_u16(self, _v: u16) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_u32(self, _v: u32) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_u64(self, _v: u64) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_i128(self, _v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_u128(self, _v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_f32(self, _v: f32) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_f64(self, _v: f64) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_char(self, _v: char) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_str(self, _v: &str) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_bytes(self, _v: &[u8]) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_none(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(
        self,
        _name: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        self.0.find_or_add(variant);
        Ok(())
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.0.find_or_add(variant);
        value.serialize(self)
    }

    #[inline]
    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn serialize_tuple(
        self,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTupleVariant, Self::Error> {
        self.0.find_or_add(variant);
        Ok(self)
    }

    #[inline]
    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeMap, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeStructVariant, Self::Error> {
        self.0.find_or_add(variant);
        Ok(self)
    }
}

impl serde::ser::SerializeMap for &mut SymbolMapPopulator<'_> {
    type Error = SymbolMapPopulationError;
    type Ok = ();

    #[inline]
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        key.serialize(&mut SymbolMapPopulator(&mut *self.0))
    }

    #[inline]
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut SymbolMapPopulator(&mut *self.0))
    }

    #[inline]
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl serde::ser::SerializeSeq for &mut SymbolMapPopulator<'_> {
    type Error = SymbolMapPopulationError;
    type Ok = ();

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut SymbolMapPopulator(&mut *self.0))
    }

    #[inline]
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl serde::ser::SerializeStruct for &mut SymbolMapPopulator<'_> {
    type Error = SymbolMapPopulationError;
    type Ok = ();

    #[inline]
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.0.find_or_add(key);
        value.serialize(&mut SymbolMapPopulator(&mut *self.0))
    }

    #[inline]
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl serde::ser::SerializeStructVariant for &mut SymbolMapPopulator<'_> {
    type Error = SymbolMapPopulationError;
    type Ok = ();

    #[inline]
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.0.find_or_add(key);
        value.serialize(&mut SymbolMapPopulator(&mut *self.0))
    }

    #[inline]
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl serde::ser::SerializeTuple for &mut SymbolMapPopulator<'_> {
    type Error = SymbolMapPopulationError;
    type Ok = ();

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut SymbolMapPopulator(&mut *self.0))
    }

    #[inline]
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
impl serde::ser::SerializeTupleStruct for &mut SymbolMapPopulator<'_> {
    type Error = SymbolMapPopulationError;
    type Ok = ();

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut SymbolMapPopulator(&mut *self.0))
    }

    #[inline]
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
impl serde::ser::SerializeTupleVariant for &mut SymbolMapPopulator<'_> {
    type Error = SymbolMapPopulationError;
    type Ok = ();

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut SymbolMapPopulator(&mut *self.0))
    }

    #[inline]
    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// A [`Serialize`] implementation returned an error.
#[derive(Debug)]
pub struct SymbolMapPopulationError(String);

impl Display for SymbolMapPopulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SymbolMapPopulationError {}

impl serde::ser::Error for SymbolMapPopulationError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self(msg.to_string())
    }
}

struct SymbolIdSorter<'a, T, F> {
    source: &'a [T],
    map: F,
    min: usize,
    id: u32,
}

impl<'a, T, F> SymbolIdSorter<'a, T, F>
where
    F: FnMut(&T) -> u32,
{
    pub fn new(source: &'a [T], map: F) -> Self {
        Self {
            source,
            map,
            min: 0,
            id: 0,
        }
    }
}
impl<'a, T, F> Iterator for SymbolIdSorter<'a, T, F>
where
    F: FnMut(&T) -> u32,
    T: Clone,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let mut encountered_greater = false;
        let start_min = self.min;
        for (relative_index, entry) in self.source[start_min..].iter().enumerate() {
            let id = (self.map)(entry);
            match id.cmp(&self.id) {
                Ordering::Equal => {
                    self.id += 1;
                    let index = start_min + relative_index;
                    if !encountered_greater {
                        self.min = index + 1;
                    }
                    return Some(index);
                }
                Ordering::Greater => encountered_greater = true,
                Ordering::Less if !encountered_greater => self.min = start_min + relative_index,
                Ordering::Less => {}
            }
        }

        None
    }
}

#[test]
fn symbol_map_debug() {
    let mut map = EphemeralSymbolMap::default();
    // To force the order, we're splitting a single string into multiple parts.
    let full_source = "abcd";

    map.find_or_add(&full_source[1..2]);
    map.find_or_add(&full_source[0..1]);
    map.find_or_add(&full_source[2..3]);
    map.find_or_add(&full_source[3..4]);

    // Verify the map sorted the symbols correctly (by memory address).
    assert_eq!(map.symbols[0].0, "a");
    assert_eq!(map.symbols[1].0, "b");
    assert_eq!(map.symbols[2].0, "c");
    assert_eq!(map.symbols[3].0, "d");

    // Verify the debug output printed the correct order.
    assert_eq!(format!("{map:?}"), r#"{"b", "a", "c", "d"}"#);
}
