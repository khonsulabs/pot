#![doc = include_str!("../crate-docs.md")]
#![forbid(unsafe_code)]
#![warn(
    clippy::cargo,
    missing_docs,
    // clippy::missing_docs_in_private_items,
    clippy::pedantic,
    future_incompatible,
    rust_2018_idioms,
)]
#![allow(
    clippy::missing_errors_doc, // TODO clippy::missing_errors_doc
    clippy::option_if_let_else,
    clippy::used_underscore_binding, // false positive with tracing
    clippy::module_name_repetitions,
)]

/// Types for deserializing pots.
pub mod de;
mod error;
/// Low-level interface for reading and writing the pot format.
pub mod format;
/// Types for reading data.
pub mod reader;
/// Types for serializing pots.
pub mod ser;
mod value;
use std::io::Read;

use byteorder::WriteBytesExt;

pub use self::error::Error;
pub use self::value::{OwnedValue, Value, ValueError, ValueIter};
/// A result alias that returns [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::de::SymbolMapRef;
use crate::reader::IoReader;

/// Serialize `value` using Pot into a `Vec<u8>`.
///
/// ```rust
/// let serialized = pot::to_vec(&"hello world").unwrap();
/// let deserialized = pot::from_slice::<String>(&serialized).unwrap();
/// assert_eq!(deserialized, "hello world");
/// ```
#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    Config::default().serialize(value)
}

/// Serialize `value` using Pot into `writer`.
///
/// ```rust
/// let mut serialized = Vec::new();
/// pot::to_writer(&"hello world", &mut serialized).unwrap();
/// let deserialized = pot::from_reader::<String, _>(&serialized[..]).unwrap();
/// assert_eq!(deserialized, "hello world");
/// ```
#[inline]
pub fn to_writer<T, W>(value: &T, writer: W) -> Result<()>
where
    T: Serialize,
    W: WriteBytesExt,
{
    Config::default().serialize_into(value, writer)
}

/// Restores a previously Pot-serialized value from a slice.
///
/// ```rust
/// let serialized = pot::to_vec(&"hello world").unwrap();
/// let deserialized = pot::from_slice::<String>(&serialized).unwrap();
/// assert_eq!(deserialized, "hello world");
/// ```
#[inline]
pub fn from_slice<'a, T>(serialized: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    Config::default().deserialize(serialized)
}

/// Restores a previously Pot-serialized value from a [`Read`] implementer.
///
/// ```rust
/// let mut serialized = Vec::new();
/// pot::to_writer(&"hello world", &mut serialized).unwrap();
/// let deserialized = pot::from_reader::<String, _>(&serialized[..]).unwrap();
/// assert_eq!(deserialized, "hello world");
/// ```
#[inline]
pub fn from_reader<T, R>(reader: R) -> Result<T>
where
    T: DeserializeOwned,
    R: Read,
{
    Config::default().deserialize_from(reader)
}

/// Serialization and deserialization configuration.
#[must_use]
#[derive(Clone, Debug)]
pub struct Config {
    allocation_budget: usize,
    compatibility: Compatibility,
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Returns the default configuration.
    pub const fn new() -> Self {
        Self {
            allocation_budget: usize::MAX,
            compatibility: Compatibility::const_default(),
        }
    }
    /// Sets the maximum number of bytes able to be allocated. This is not
    /// guaranteed to be perfectly accurate, due to the limitations of serde
    /// deserializers. Pot can keep track of how many bytes it thinks its
    /// allocating, but a deserializer can always allocate more memory than Pot
    /// can be aware of.
    ///
    /// The default allocation budget is [`usize::MAX`].
    #[inline]
    pub const fn allocation_budget(mut self, budget: usize) -> Self {
        self.allocation_budget = budget;
        self
    }

    /// Sets the compatibility mode for serializing and returns self.
    pub const fn compatibility(mut self, compatibilty: Compatibility) -> Self {
        self.compatibility = compatibilty;
        self
    }

    /// Deserializes a value from a slice using the configured options.
    #[inline]
    pub fn deserialize<'de, T>(&self, serialized: &'de [u8]) -> Result<T>
    where
        T: Deserialize<'de>,
    {
        let mut deserializer = de::Deserializer::from_slice(serialized, self.allocation_budget)?;
        let t = T::deserialize(&mut deserializer)?;
        if deserializer.end_of_input() {
            Ok(t)
        } else {
            Err(Error::TrailingBytes)
        }
    }

    /// Deserializes a value from a [`Read`] implementer using the configured
    /// options.
    #[inline]
    pub fn deserialize_from<T, R: Read>(&self, reader: R) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut deserializer = de::Deserializer::from_read(
            IoReader::new(reader),
            SymbolMapRef::temporary(),
            self.allocation_budget,
        )?;
        T::deserialize(&mut deserializer)
    }

    /// Serializes a value to a `Vec` using the configured options.
    #[inline]
    pub fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        self.serialize_into(value, &mut output)?;
        Ok(output)
    }

    /// Serializes a value to a writer using the configured options.
    #[allow(clippy::unused_self)]
    #[inline]
    pub fn serialize_into<T, W>(&self, value: &T, writer: W) -> Result<()>
    where
        T: Serialize,
        W: WriteBytesExt,
    {
        let mut serializer = ser::Serializer::new_with_compatibility(writer, self.compatibility)?;
        value.serialize(&mut serializer)
    }
}

/// Compatibility settings for Pot.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[non_exhaustive]
pub enum Compatibility {
    /// Serializes data that is compatible with all versions of Pot
    /// deserializers.
    ///
    /// This format does not support [`Value`](crate::Value) deserialization of
    /// enum variants without associated data. See [`V5`](Self::V5) for more
    /// information.
    Full,
    /// Serializes data in the default format
    ///
    /// This format has a single change in how enum variants without associated
    /// data are serialized. This change allows `deserialize_any` to
    /// unambiguously distinguish between variants with associated data and
    /// variants without associated data.
    ///
    /// This will be the default compatibility setting in `v4.0` and later. All
    /// versions after `v3.0.1` are able to read this updated format.
    V4,
}

impl Compatibility {
    const fn const_default() -> Self {
        Self::Full
    }
}

impl Default for Compatibility {
    fn default() -> Self {
        Self::const_default()
    }
}

#[cfg(test)]
mod tests;
