use std::{
    borrow::Cow,
    fmt::{Display, Write},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use serde::{
    de::{EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor},
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Deserialize, Serialize,
};

use crate::format::{Float, InnerFloat, InnerInteger, Integer};

/// A Pot encoded value. This type can be used to deserialize to and from Pot
/// without knowing the original data structure.
#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    /// A value representing None.
    None,
    /// A value representing a Unit (`()`).
    Unit,
    /// A boolean value
    Bool(bool),
    /// An integer value.
    Integer(Integer),
    /// A floating point value.
    Float(Float),
    /// A value containing arbitrary bytes.
    Bytes(Cow<'a, [u8]>),
    /// A string value.
    String(Cow<'a, str>),
    /// A sequence of values.
    Sequence(Vec<Self>),
    /// A sequence of key-value mappings.
    Mappings(Vec<(Self, Self)>),
}

impl<'a> Display for Value<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::None => f.write_str("None"),
            Value::Unit => f.write_str("()"),
            Value::Bool(true) => f.write_str("true"),
            Value::Bool(false) => f.write_str("false"),
            Value::Integer(value) => Display::fmt(value, f),
            Value::Float(value) => Display::fmt(value, f),
            Value::Bytes(bytes) => {
                f.write_str("0x")?;
                for (index, byte) in bytes.iter().enumerate() {
                    if index > 0 && index % 4 == 0 {
                        f.write_char('_')?;
                    }
                    write!(f, "{byte:02x}")?;
                }
                Ok(())
            }
            Value::String(string) => f.write_str(string),
            Value::Sequence(sequence) => {
                f.write_char('[')?;
                for (index, value) in sequence.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    Display::fmt(value, f)?;
                }
                f.write_char(']')
            }
            Value::Mappings(mappings) => {
                f.write_char('{')?;
                for (index, (key, value)) in mappings.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }
                    Display::fmt(key, f)?;
                    f.write_str(": ")?;
                    Display::fmt(value, f)?;
                }
                f.write_char('}')
            }
        }
    }
}

impl<'a> Value<'a> {
    /// Creates a `Value` from the given serde-compatible type.
    ///
    /// ```rust
    /// use pot::Value;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize, Debug)]
    /// enum Example {
    ///     Hello,
    ///     World
    /// }
    ///
    ///
    /// let original = vec![Example::Hello, Example::World];
    /// let serialized = Value::from_serialize(&original);
    /// assert_eq!(
    ///     serialized,
    ///     Value::Sequence(vec![
    ///         Value::from(String::from("Hello")), Value::from(String::from("World"))
    ///     ])
    /// );
    /// ```
    pub fn from_serialize<T: Serialize>(value: T) -> Self {
        let Ok(value) = value.serialize(Serializer) else { unreachable!() };
        value
    }

    /// Attempts to create an instance of `T` from this value.
    ///
    /// ```rust
    /// use pot::Value;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    /// enum Example {
    ///     Hello,
    ///     World
    /// }
    ///
    /// let original = vec![Example::Hello, Example::World];
    /// let serialized = Value::from_serialize(&original);
    /// let deserialized: Vec<Example> = serialized.deserialize_as().unwrap();
    /// assert_eq!(deserialized, original);
    /// ```
    pub fn deserialize_as<'de, T: Deserialize<'de>>(&'de self) -> Result<T, ValueError> {
        T::deserialize(Deserializer(self))
    }

    /// Returns a new value from an interator of items that can be converted into a value.
    ///
    /// ```rust
    /// # use pot::Value;
    /// let mappings = Value::from_sequence(Vec::<String>::new());
    /// assert!(matches!(mappings, Value::Sequence(_)));
    /// ```
    pub fn from_sequence<IntoIter: IntoIterator<Item = T>, T: Into<Self>>(
        sequence: IntoIter,
    ) -> Self {
        Self::Sequence(sequence.into_iter().map(T::into).collect())
    }

    /// Returns a new value from an interator of 2-element tuples representing key-value pairs.
    ///
    /// ```rust
    /// # use pot::Value;
    /// # use std::collections::HashMap;
    /// let mappings = Value::from_mappings(HashMap::<String, u32>::new());
    /// assert!(matches!(mappings, Value::Mappings(_)));
    /// ```
    pub fn from_mappings<IntoIter: IntoIterator<Item = (K, V)>, K: Into<Self>, V: Into<Self>>(
        mappings: IntoIter,
    ) -> Self {
        Self::Mappings(
            mappings
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }

    /// Returns true if the value contained is considered empty.
    ///
    /// ```rust
    /// # use pot::Value;
    /// // Value::None is always empty.
    /// assert_eq!(Value::None.is_empty(), true);
    ///
    /// // All primitive values, including Unit, are always not empty, even if they contain the value 0.
    /// assert_eq!(Value::Unit.is_empty(), false);
    /// assert_eq!(Value::from(false).is_empty(), false);
    /// assert_eq!(Value::from(0_u8).is_empty(), false);
    /// assert_eq!(Value::from(0_f32).is_empty(), false);
    ///
    /// // For all other types, having a length of 0 will result in is_empty returning true.
    /// assert_eq!(Value::from(Vec::<u8>::new()).is_empty(), true);
    /// assert_eq!(Value::from(b"").is_empty(), true);
    /// assert_eq!(Value::from(vec![0_u8]).is_empty(), false);
    ///
    /// assert_eq!(Value::from("").is_empty(), true);
    /// assert_eq!(Value::from("hi").is_empty(), false);
    ///
    /// assert_eq!(Value::Sequence(Vec::new()).is_empty(), true);
    /// assert_eq!(Value::from(vec![Value::None]).is_empty(), false);
    ///
    /// assert_eq!(Value::Mappings(Vec::new()).is_empty(), true);
    /// assert_eq!(
    ///     Value::from(vec![(Value::None, Value::None)]).is_empty(),
    ///     false
    /// );
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Value::None => true,
            Value::Unit | Value::Bool(_) | Value::Integer(_) | Value::Float(_) => false,
            Value::Bytes(value) => value.is_empty(),
            Value::String(value) => value.is_empty(),
            Value::Sequence(value) => value.is_empty(),
            Value::Mappings(value) => value.is_empty(),
        }
    }

    /// Returns the value represented as a value.
    ///
    /// ```rust
    /// # use pot::Value;
    /// // Value::None is always false.
    /// assert_eq!(Value::None.as_bool(), false);
    ///
    /// // Value::Unit is always true.
    /// assert_eq!(Value::Unit.as_bool(), true);
    ///
    /// // Value::Bool will return the contained value
    /// assert_eq!(Value::from(false).as_bool(), false);
    /// assert_eq!(Value::from(true).as_bool(), true);
    ///
    /// // All primitive values return true if the value is non-zero.
    /// assert_eq!(Value::from(0_u8).as_bool(), false);
    /// assert_eq!(Value::from(1_u8).as_bool(), true);
    /// assert_eq!(Value::from(0_f32).as_bool(), false);
    /// assert_eq!(Value::from(1_f32).as_bool(), true);
    ///
    /// // For all other types, as_bool() returns the result of `!is_empty()`.
    /// assert_eq!(Value::from(Vec::<u8>::new()).as_bool(), false);
    /// assert_eq!(Value::from(b"").as_bool(), false);
    /// assert_eq!(Value::from(vec![0_u8]).as_bool(), true);
    ///
    /// assert_eq!(Value::from("").as_bool(), false);
    /// assert_eq!(Value::from("hi").as_bool(), true);
    ///
    /// assert_eq!(Value::Sequence(Vec::new()).as_bool(), false);
    /// assert_eq!(Value::from(vec![Value::None]).as_bool(), true);
    ///
    /// assert_eq!(Value::Mappings(Vec::new()).as_bool(), false);
    /// assert_eq!(
    ///     Value::from(vec![(Value::None, Value::None)]).as_bool(),
    ///     true
    /// );
    /// ```
    #[must_use]
    pub fn as_bool(&self) -> bool {
        match self {
            Value::None => false,
            Value::Unit => true,
            Value::Bool(value) => *value,
            Value::Integer(value) => !value.is_zero(),
            Value::Float(value) => !value.is_zero(),
            Value::Bytes(value) => !value.is_empty(),
            Value::String(value) => !value.is_empty(),
            Value::Sequence(value) => !value.is_empty(),
            Value::Mappings(value) => !value.is_empty(),
        }
    }

    /// Returns the value as an [`Integer`]. Returns None if the value is not a
    /// [`Self::Float`] or [`Self::Integer`]. Also returns None if the value is
    /// a float, but cannot be losslessly converted to an integer.
    #[must_use]
    pub fn as_integer(&self) -> Option<Integer> {
        match self {
            Value::Integer(value) => Some(*value),
            Value::Float(value) => value.as_integer().ok(),
            _ => None,
        }
    }

    /// Returns the value as an [`Float`]. Returns None if the value is not a
    /// [`Self::Float`] or [`Self::Integer`]. Also returns None if the value is
    /// an integer, but cannot be losslessly converted to a float.
    #[must_use]
    pub fn as_float(&self) -> Option<Float> {
        match self {
            Value::Integer(value) => value.as_float().ok(),
            Value::Float(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the value as a string, or None if the value is not representable
    /// by a string. This will only return a value with variants
    /// [`Self::String`] and [`Self::Bytes`]. Bytes will only be returned if the
    /// contained bytes can be safely interpretted as utf-8.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Bytes(bytes) => std::str::from_utf8(bytes).ok(),
            Self::String(string) => Some(string),
            _ => None,
        }
    }

    /// Returns the value's bytes, or None if the value is not stored as a
    /// representation of bytes. This will only return a value with variants
    /// [`Self::String`] and [`Self::Bytes`].
    #[must_use]
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(bytes) => Some(bytes),
            Self::String(string) => Some(string.as_bytes()),
            _ => None,
        }
    }

    /// Returns an interator that iterates over all values contained inside of
    /// this value. Returns an empty iterator if not a [`Self::Sequence`] or
    /// [`Self::Mappings`]. If a [`Self::Mappings`], only the value portion of
    /// the mapping is returned.
    #[must_use]
    pub fn values(&self) -> SequenceIter<'_> {
        match self {
            Self::Sequence(sequence) => SequenceIter::Sequence(sequence.iter()),
            Self::Mappings(mappings) => SequenceIter::Mappings(mappings.iter()),

            _ => SequenceIter::Sequence([].iter()),
        }
    }

    /// Returns an interator that iterates over all mappings contained inside of
    /// this value. Returns an empty iterator if not a [`Self::Sequence`] or
    /// [`Self::Mappings`]. If a [`Self::Sequence`], the key will always be
    /// `Self::None`.
    pub fn mappings(&self) -> std::slice::Iter<'_, (Self, Self)> {
        match self {
            Self::Mappings(mappings) => mappings.iter(),
            _ => [].iter(),
        }
    }

    /// Converts `self` to a static lifetime by cloning any borrowed data.
    pub fn into_static(self) -> Value<'static> {
        match self {
            Self::None => Value::None,
            Self::Unit => Value::Unit,
            Self::Bool(value) => Value::Bool(value),
            Self::Integer(value) => Value::Integer(value),
            Self::Float(value) => Value::Float(value),
            Self::Bytes(Cow::Owned(value)) => Value::Bytes(Cow::Owned(value)),
            Self::Bytes(Cow::Borrowed(value)) => Value::Bytes(Cow::Owned(value.to_vec())),
            Self::String(Cow::Owned(value)) => Value::String(Cow::Owned(value)),
            Self::String(Cow::Borrowed(value)) => Value::String(Cow::Owned(value.to_string())),
            Self::Sequence(value) => {
                Value::Sequence(value.into_iter().map(Value::into_static).collect())
            }
            Self::Mappings(value) => Value::Mappings(
                value
                    .into_iter()
                    .map(|(k, v)| (k.into_static(), v.into_static()))
                    .collect(),
            ),
        }
    }

    /// Converts `self` to a static lifetime by cloning all data.
    pub fn to_static(&self) -> Value<'static> {
        match self {
            Self::None => Value::None,
            Self::Unit => Value::Unit,
            Self::Bool(value) => Value::Bool(*value),
            Self::Integer(value) => Value::Integer(*value),
            Self::Float(value) => Value::Float(*value),
            Self::Bytes(Cow::Owned(value)) => Value::Bytes(Cow::Owned(value.clone())),
            Self::Bytes(Cow::Borrowed(value)) => Value::Bytes(Cow::Owned(value.to_vec())),
            Self::String(Cow::Owned(value)) => Value::String(Cow::Owned(value.clone())),
            Self::String(Cow::Borrowed(value)) => Value::String(Cow::Owned((*value).to_string())),
            Self::Sequence(value) => Value::Sequence(value.iter().map(Value::to_static).collect()),
            Self::Mappings(value) => Value::Mappings(
                value
                    .iter()
                    .map(|(k, v)| (k.to_static(), v.to_static()))
                    .collect(),
            ),
        }
    }
}

impl<'a> Serialize for Value<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::None => serializer.serialize_none(),
            Value::Unit => serializer.serialize_unit(),
            Value::Bool(value) => serializer.serialize_bool(*value),
            Value::Integer(integer) => match integer.0 {
                InnerInteger::I8(value) => serializer.serialize_i8(value),
                InnerInteger::I16(value) => serializer.serialize_i16(value),
                InnerInteger::I32(value) => serializer.serialize_i32(value),
                InnerInteger::I64(value) => serializer.serialize_i64(value),
                InnerInteger::I128(value) => serializer.serialize_i128(value),
                InnerInteger::U8(value) => serializer.serialize_u8(value),
                InnerInteger::U16(value) => serializer.serialize_u16(value),
                InnerInteger::U32(value) => serializer.serialize_u32(value),
                InnerInteger::U64(value) => serializer.serialize_u64(value),
                InnerInteger::U128(value) => serializer.serialize_u128(value),
            },
            Value::Float(value) => match value.0 {
                InnerFloat::F64(value) => serializer.serialize_f64(value),
                InnerFloat::F32(value) => serializer.serialize_f32(value),
            },
            Value::Bytes(value) => serializer.serialize_bytes(value),
            Value::String(value) => serializer.serialize_str(value),
            Value::Sequence(values) => {
                let mut seq = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    seq.serialize_element(value)?;
                }
                seq.end()
            }
            Value::Mappings(keys_and_values) => {
                let mut map = serializer.serialize_map(Some(keys_and_values.len()))?;
                for (key, value) in keys_and_values {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for Value<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor::default())
    }
}

/// A [`Value<'static>`] wrapper that supports
/// [`DeserializeOwned`](serde::DeserializeOwned).
///
/// Because `Value<'a>` can borrow strings and bytes during deserialization,
/// `Value<'static>` can't be used when `DeserializeOwned` is needed.
/// [`OwnedValue`] implements [`Deserialize`] by first deserializing a
/// `Value<'a>` and then using [`Value::into_static`] to convert borrowed data
/// to owned data.
#[derive(Debug, Clone, PartialEq)]
pub struct OwnedValue(pub Value<'static>);

impl Deref for OwnedValue {
    type Target = Value<'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OwnedValue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Serialize for OwnedValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for OwnedValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_any(ValueVisitor::default())
            .map(|value| Self(value.into_static()))
    }
}

#[derive(Default)]
struct ValueVisitor<'a>(PhantomData<&'a ()>);

impl<'de: 'a, 'a> Visitor<'de> for ValueVisitor<'a> {
    type Value = Value<'a>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("any value")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::None)
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Bool(v))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Float(Float::from(v)))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Float(Float::from(v)))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::String(Cow::Owned(v.to_string())))
    }

    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::String(Cow::Borrowed(v)))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::String(Cow::Owned(v)))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Bytes(Cow::Owned(v.to_vec())))
    }

    fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Bytes(Cow::Borrowed(v)))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Bytes(Cow::Owned(v)))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(Self::default())
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Unit)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut values = if let Some(hint) = seq.size_hint() {
            Vec::with_capacity(hint)
        } else {
            Vec::new()
        };
        while let Some(value) = seq.next_element()? {
            values.push(value);
        }
        Ok(Value::Sequence(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut values = if let Some(hint) = map.size_hint() {
            Vec::with_capacity(hint)
        } else {
            Vec::new()
        };
        while let Some(value) = map.next_entry()? {
            values.push(value);
        }
        Ok(Value::Mappings(values))
    }
}

impl<'a> From<Option<Value<'a>>> for Value<'a> {
    fn from(value: Option<Value<'a>>) -> Self {
        if let Some(value) = value {
            value
        } else {
            Value::None
        }
    }
}

impl<'a> From<()> for Value<'a> {
    fn from(_: ()) -> Self {
        Value::Unit
    }
}

impl<'a> From<bool> for Value<'a> {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

macro_rules! define_value_from_primitive {
    ($container:ident, $variant:ident, $primitive:ty) => {
        impl<'a> From<$primitive> for Value<'a> {
            fn from(value: $primitive) -> Self {
                Self::$container($container::from(value))
            }
        }
    };
}

define_value_from_primitive!(Integer, U8, u8);
define_value_from_primitive!(Integer, U16, u16);
define_value_from_primitive!(Integer, U32, u32);
define_value_from_primitive!(Integer, U64, u64);
define_value_from_primitive!(Integer, U128, u128);

define_value_from_primitive!(Integer, I8, i8);
define_value_from_primitive!(Integer, I16, i16);
define_value_from_primitive!(Integer, I32, i32);
define_value_from_primitive!(Integer, I64, i64);
define_value_from_primitive!(Integer, I128, i128);

define_value_from_primitive!(Float, F32, f32);
define_value_from_primitive!(Float, F64, f64);

impl<'a> From<&'a [u8]> for Value<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self::Bytes(Cow::Borrowed(bytes))
    }
}

impl<'a> From<Vec<u8>> for Value<'a> {
    fn from(bytes: Vec<u8>) -> Self {
        Self::Bytes(Cow::Owned(bytes))
    }
}

impl<'a, const N: usize> From<&'a [u8; N]> for Value<'a> {
    fn from(bytes: &'a [u8; N]) -> Self {
        Self::Bytes(Cow::Borrowed(bytes))
    }
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(string: &'a str) -> Self {
        Self::String(Cow::Borrowed(string))
    }
}

impl<'a> From<String> for Value<'a> {
    fn from(string: String) -> Self {
        Self::String(Cow::Owned(string))
    }
}

impl<'a> From<Vec<Value<'a>>> for Value<'a> {
    fn from(value: Vec<Value<'a>>) -> Self {
        Self::Sequence(value)
    }
}

impl<'a> From<Vec<(Value<'a>, Value<'a>)>> for Value<'a> {
    fn from(value: Vec<(Value<'a>, Value<'a>)>) -> Self {
        Self::Mappings(value)
    }
}

pub enum SequenceIter<'a> {
    Sequence(std::slice::Iter<'a, Value<'a>>),
    Mappings(std::slice::Iter<'a, (Value<'a>, Value<'a>)>),
}

impl<'a> Iterator for SequenceIter<'a> {
    type Item = &'a Value<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SequenceIter::Sequence(sequence) => sequence.next(),
            SequenceIter::Mappings(mappings) => mappings.next().map(|(_k, v)| v),
        }
    }
}

#[test]
#[allow(clippy::cognitive_complexity)]
fn value_display_tests() {
    // Specials
    assert_eq!(Value::None.to_string(), "None");
    assert_eq!(Value::Unit.to_string(), "()");

    // Boolean
    assert_eq!(Value::Bool(false).to_string(), "false");
    assert_eq!(Value::Bool(true).to_string(), "true");

    // Integer
    assert_eq!(Value::from(1_u8).to_string(), "1");
    assert_eq!(Value::from(1_u16).to_string(), "1");
    assert_eq!(Value::from(1_u32).to_string(), "1");
    assert_eq!(Value::from(1_u64).to_string(), "1");
    assert_eq!(Value::from(1_u128).to_string(), "1");
    assert_eq!(Value::from(1_i8).to_string(), "1");
    assert_eq!(Value::from(1_i16).to_string(), "1");
    assert_eq!(Value::from(1_i32).to_string(), "1");
    assert_eq!(Value::from(1_i64).to_string(), "1");
    assert_eq!(Value::from(1_i128).to_string(), "1");

    // Float
    assert_eq!(Value::from(1.1_f32).to_string(), "1.1");
    assert_eq!(Value::from(1.1_f64).to_string(), "1.1");

    // Bytes
    assert_eq!(Value::from(b"\xFE\xED\xD0\xD0").to_string(), "0xfeedd0d0");
    assert_eq!(
        Value::from(b"\xFE\xED\xD0\xD0\xDE\xAD\xBE\xEF").to_string(),
        "0xfeedd0d0_deadbeef"
    );

    // String
    assert_eq!(Value::from("hello world").to_string(), "hello world");

    // Sequence
    assert_eq!(
        Value::from_sequence(Vec::<Value<'_>>::new()).to_string(),
        "[]"
    );
    assert_eq!(
        Value::from_sequence(vec![Value::None]).to_string(),
        "[None]"
    );
    assert_eq!(
        Value::from_sequence(vec![Value::None, Value::Unit]).to_string(),
        "[None, ()]"
    );

    // Mappings
    assert_eq!(
        Value::from_mappings(Vec::<(Value<'_>, Value<'_>)>::new()).to_string(),
        "{}"
    );
    assert_eq!(
        Value::from_mappings(vec![(Value::from(0_u8), Value::None)]).to_string(),
        "{0: None}"
    );
    assert_eq!(
        Value::from_mappings(vec![
            (Value::from(0_u8), Value::None),
            (Value::from(1_u8), Value::Unit)
        ])
        .to_string(),
        "{0: None, 1: ()}"
    );
}

#[test]
#[allow(clippy::manual_assert)] // approx::assert_relative_eq false positive
fn value_as_float_tests() {
    approx::assert_relative_eq!(
        Value::from(u8::MAX)
            .as_float()
            .expect("u8 conversion failed")
            .as_f32()
            .expect("f32 conversion failed"),
        255_f32,
    );
    approx::assert_relative_eq!(
        Value::from(u32::MAX)
            .as_float()
            .expect("u32 conversion failed")
            .as_f64(),
        4_294_967_295_f64,
    );
    assert!(Value::from(u32::MAX)
        .as_float()
        .expect("u32 conversion failed")
        .as_f32()
        .is_err());

    approx::assert_relative_eq!(Value::from(0_f64).as_float().unwrap().as_f32().unwrap(), 0.);
}

#[test]
fn value_as_integer_tests() {
    macro_rules! test_signed {
        ($primitive:ty, $signed_method:ident, $unsigned:ty, $unsigned_method:ident, $float:ty) => {
            assert_eq!(
                Value::from(<$float>::from(<$primitive>::MAX))
                    .as_integer()
                    .expect("integer conversion failed")
                    .$signed_method()
                    .unwrap(),
                <$primitive>::MAX,
            );
            assert_eq!(
                Value::from(<$float>::from(<$primitive>::MIN))
                    .as_integer()
                    .expect("integer conversion failed")
                    .$signed_method()
                    .unwrap(),
                <$primitive>::MIN,
            );
            assert_eq!(
                Value::from(<$float>::from(<$primitive>::MAX))
                    .as_integer()
                    .expect("integer conversion failed")
                    .$unsigned_method()
                    .unwrap(),
                <$unsigned>::try_from(<$primitive>::MAX).unwrap(),
            );
        };
    }

    test_signed!(i8, as_i8, u8, as_u8, f32);
    test_signed!(i16, as_i16, u16, as_u16, f32);
    test_signed!(i32, as_i32, u32, as_u32, f64);

    macro_rules! test_unsigned {
        ($primitive:ty, $unsigned_method:ident, $signed:ty, $signed_method:ident, $float:ty) => {
            assert_eq!(
                Value::from(<$float>::from(<$primitive>::MAX))
                    .as_integer()
                    .expect("integer conversion failed")
                    .$unsigned_method()
                    .unwrap(),
                <$primitive>::MAX,
            );
            assert!(Value::from(<$float>::from(<$primitive>::MAX))
                .as_integer()
                .expect("integer conversion failed")
                .$signed_method()
                .is_err());
        };
    }

    test_unsigned!(u8, as_u8, i8, as_i8, f32);
    test_unsigned!(u16, as_u16, i16, as_i16, f32);
    test_unsigned!(u32, as_u32, i32, as_i32, f64);
}

struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = Value<'static>;

    type Error = Infallible;
    type SerializeSeq = SequenceSerializer;
    type SerializeTuple = SequenceSerializer;
    type SerializeTupleStruct = SequenceSerializer;
    type SerializeTupleVariant = TupleVariantSerializer;
    type SerializeMap = MappingsSerializer;
    type SerializeStruct = MappingsSerializer;
    type SerializeStructVariant = StructVariantSerializer;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Integer(Integer::from(v)))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Float(Float::from(v)))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Float(Float::from(v)))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Integer(Integer::from(u32::from(v))))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(Cow::Owned(v.to_string())))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bytes(Cow::Owned(v.to_vec())))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::None)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(Self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Unit)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Unit)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(Cow::Borrowed(variant)))
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(Self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Ok(Value::Mappings(vec![(
            Value::String(Cow::Borrowed(variant)),
            value.serialize(Self)?,
        )]))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SequenceSerializer(
            len.map_or_else(Vec::new, Vec::with_capacity),
        ))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SequenceSerializer(Vec::with_capacity(len)))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(SequenceSerializer(Vec::with_capacity(len)))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(TupleVariantSerializer {
            variant,
            sequence: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MappingsSerializer(
            len.map_or_else(Vec::new, Vec::with_capacity),
        ))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(MappingsSerializer(Vec::with_capacity(len)))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(StructVariantSerializer {
            variant,
            mappings: Vec::with_capacity(len),
        })
    }
}

struct SequenceSerializer(Vec<Value<'static>>);

impl SerializeSeq for SequenceSerializer {
    type Ok = Value<'static>;
    type Error = Infallible;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.0.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Sequence(self.0))
    }
}

impl SerializeTuple for SequenceSerializer {
    type Ok = Value<'static>;
    type Error = Infallible;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.0.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Sequence(self.0))
    }
}

impl SerializeTupleStruct for SequenceSerializer {
    type Ok = Value<'static>;
    type Error = Infallible;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.0.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Sequence(self.0))
    }
}

struct TupleVariantSerializer {
    variant: &'static str,
    sequence: Vec<Value<'static>>,
}

impl SerializeTupleVariant for TupleVariantSerializer {
    type Ok = Value<'static>;
    type Error = Infallible;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.sequence.push(value.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Mappings(vec![(
            Value::String(Cow::Borrowed(self.variant)),
            Value::Sequence(self.sequence),
        )]))
    }
}

struct MappingsSerializer(Vec<(Value<'static>, Value<'static>)>);

impl SerializeMap for MappingsSerializer {
    type Ok = Value<'static>;
    type Error = Infallible;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.0.push((key.serialize(Serializer)?, Value::None));
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.0
            .last_mut()
            .expect("serialize_value called without serialize_key")
            .1 = value.serialize(Serializer)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Mappings(self.0))
    }
}

impl SerializeStruct for MappingsSerializer {
    type Ok = Value<'static>;
    type Error = Infallible;
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.0.push((
            Value::String(Cow::Borrowed(key)),
            value.serialize(Serializer)?,
        ));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Mappings(self.0))
    }
}

struct StructVariantSerializer {
    variant: &'static str,
    mappings: Vec<(Value<'static>, Value<'static>)>,
}

impl SerializeStructVariant for StructVariantSerializer {
    type Ok = Value<'static>;
    type Error = Infallible;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.mappings.push((
            Value::String(Cow::Borrowed(key)),
            value.serialize(Serializer)?,
        ));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Mappings(vec![(
            Value::String(Cow::Borrowed(self.variant)),
            Value::Mappings(self.mappings),
        )]))
    }
}

struct Deserializer<'de>(&'de Value<'de>);

impl<'de> serde::Deserializer<'de> for Deserializer<'de> {
    type Error = ValueError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match &self.0 {
            Value::None => visitor.visit_none(),
            Value::Unit => visitor.visit_unit(),
            Value::Bool(value) => visitor.visit_bool(*value),
            Value::Integer(integer) => match integer.0 {
                InnerInteger::I8(value) => visitor.visit_i8(value),
                InnerInteger::I16(value) => visitor.visit_i16(value),
                InnerInteger::I32(value) => visitor.visit_i32(value),
                InnerInteger::I64(value) => visitor.visit_i64(value),
                InnerInteger::I128(value) => visitor.visit_i128(value),
                InnerInteger::U8(value) => visitor.visit_u8(value),
                InnerInteger::U16(value) => visitor.visit_u16(value),
                InnerInteger::U32(value) => visitor.visit_u32(value),
                InnerInteger::U64(value) => visitor.visit_u64(value),
                InnerInteger::U128(value) => visitor.visit_u128(value),
            },
            Value::Float(float) => match float.0 {
                InnerFloat::F64(value) => visitor.visit_f64(value),
                InnerFloat::F32(value) => visitor.visit_f32(value),
            },
            Value::Bytes(bytes) => visitor.visit_bytes(bytes),
            Value::String(str) => visitor.visit_str(str),
            Value::Sequence(seq) => visitor.visit_seq(SequenceDeserializer(seq)),
            Value::Mappings(mappings) => visitor.visit_map(MappingsDeserializer(mappings)),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Bool(value) = &self.0 {
            visitor.visit_bool(*value)
        } else {
            Err(ValueError::Expected {
                kind: "bool",
                value: self.0.to_static(),
            })
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Integer(value) = &self.0 {
            if let Ok(value) = value.as_i8() {
                return visitor.visit_i8(value);
            }
        }

        Err(ValueError::Expected {
            kind: "i8",
            value: self.0.to_static(),
        })
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Integer(value) = &self.0 {
            if let Ok(value) = value.as_i16() {
                return visitor.visit_i16(value);
            }
        }

        Err(ValueError::Expected {
            kind: "i16",
            value: self.0.to_static(),
        })
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Integer(value) = &self.0 {
            if let Ok(value) = value.as_i32() {
                return visitor.visit_i32(value);
            }
        }

        Err(ValueError::Expected {
            kind: "i32",
            value: self.0.to_static(),
        })
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Integer(value) = &self.0 {
            if let Ok(value) = value.as_i64() {
                return visitor.visit_i64(value);
            }
        }

        Err(ValueError::Expected {
            kind: "i64",
            value: self.0.to_static(),
        })
    }
    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Integer(value) = &self.0 {
            if let Ok(value) = value.as_i128() {
                return visitor.visit_i128(value);
            }
        }

        Err(ValueError::Expected {
            kind: "i128",
            value: self.0.to_static(),
        })
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Integer(value) = &self.0 {
            if let Ok(value) = value.as_u8() {
                return visitor.visit_u8(value);
            }
        }

        Err(ValueError::Expected {
            kind: "u8",
            value: self.0.to_static(),
        })
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Integer(value) = &self.0 {
            if let Ok(value) = value.as_u16() {
                return visitor.visit_u16(value);
            }
        }

        Err(ValueError::Expected {
            kind: "u16",
            value: self.0.to_static(),
        })
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Integer(value) = &self.0 {
            if let Ok(value) = value.as_u32() {
                return visitor.visit_u32(value);
            }
        }

        Err(ValueError::Expected {
            kind: "u32",
            value: self.0.to_static(),
        })
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Integer(value) = &self.0 {
            if let Ok(value) = value.as_u64() {
                return visitor.visit_u64(value);
            }
        }

        Err(ValueError::Expected {
            kind: "u64",
            value: self.0.to_static(),
        })
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Integer(value) = &self.0 {
            if let Ok(value) = value.as_u128() {
                return visitor.visit_u128(value);
            }
        }

        Err(ValueError::Expected {
            kind: "u128",
            value: self.0.to_static(),
        })
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Float(value) = &self.0 {
            if let Ok(value) = value.as_f32() {
                return visitor.visit_f32(value);
            }
        }

        Err(ValueError::Expected {
            kind: "f32",
            value: self.0.to_static(),
        })
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Float(value) = &self.0 {
            visitor.visit_f64(value.as_f64())
        } else {
            Err(ValueError::Expected {
                kind: "f64",
                value: self.0.to_static(),
            })
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Integer(value) = &self.0 {
            if let Ok(value) = value.as_u32() {
                if let Ok(char) = char::try_from(value) {
                    return visitor.visit_char(char);
                }
            }
        }

        Err(ValueError::Expected {
            kind: "char",
            value: self.0.to_static(),
        })
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::String(value) = &self.0 {
            visitor.visit_borrowed_str(value)
        } else {
            Err(ValueError::Expected {
                kind: "str",
                value: self.0.to_static(),
            })
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::String(value) = &self.0 {
            visitor.visit_borrowed_str(value)
        } else {
            Err(ValueError::Expected {
                kind: "String",
                value: self.0.to_static(),
            })
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Bytes(value) = &self.0 {
            visitor.visit_borrowed_bytes(value)
        } else {
            Err(ValueError::Expected {
                kind: "bytes",
                value: self.0.to_static(),
            })
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Bytes(value) = &self.0 {
            visitor.visit_borrowed_bytes(value)
        } else {
            Err(ValueError::Expected {
                kind: "byte buf",
                value: self.0.to_static(),
            })
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if matches!(&self.0, Value::None) {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Unit = &self.0 {
            visitor.visit_unit()
        } else {
            Err(ValueError::Expected {
                kind: "()",
                value: self.0.to_static(),
            })
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Unit = &self.0 {
            visitor.visit_unit()
        } else {
            Err(ValueError::Expected {
                kind: "()",
                value: self.0.to_static(),
            })
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Sequence(sequence) = &self.0 {
            visitor.visit_seq(SequenceDeserializer(sequence))
        } else {
            Err(ValueError::Expected {
                kind: "sequence",
                value: self.0.to_static(),
            })
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Sequence(sequence) = &self.0 {
            visitor.visit_seq(SequenceDeserializer(sequence))
        } else {
            Err(ValueError::Expected {
                kind: "tuple",
                value: self.0.to_static(),
            })
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Sequence(sequence) = &self.0 {
            visitor.visit_seq(SequenceDeserializer(sequence))
        } else {
            Err(ValueError::Expected {
                kind: "tuple struct",
                value: self.0.to_static(),
            })
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Mappings(sequence) = &self.0 {
            visitor.visit_map(MappingsDeserializer(sequence))
        } else {
            Err(ValueError::Expected {
                kind: "map",
                value: self.0.to_static(),
            })
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Mappings(sequence) = &self.0 {
            visitor.visit_map(MappingsDeserializer(sequence))
        } else {
            Err(ValueError::Expected {
                kind: "map",
                value: self.0.to_static(),
            })
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

impl<'de> EnumAccess<'de> for Deserializer<'de> {
    type Error = ValueError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        match &self.0 {
            Value::Mappings(mapping) => {
                if !mapping.is_empty() {
                    let variant = seed.deserialize(Deserializer(&mapping[0].0))?;
                    return Ok((variant, Deserializer(&mapping[0].1)));
                }
            }
            Value::String(_) => {
                let variant = seed.deserialize(Deserializer(self.0))?;
                return Ok((variant, Deserializer(&Value::Unit)));
            }
            _ => {}
        }

        Err(ValueError::Expected {
            kind: "enum variant",
            value: self.0.to_static(),
        })
    }
}

impl<'de> VariantAccess<'de> for Deserializer<'de> {
    type Error = ValueError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        if matches!(self.0, Value::Unit) {
            Ok(())
        } else {
            Err(ValueError::Expected {
                kind: "unit",
                value: self.0.to_static(),
            })
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Sequence(sequence) = &self.0 {
            visitor.visit_seq(SequenceDeserializer(sequence))
        } else {
            Err(ValueError::Expected {
                kind: "tuple variant",
                value: self.0.to_static(),
            })
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Value::Mappings(mappings) = &self.0 {
            visitor.visit_map(MappingsDeserializer(mappings))
        } else {
            Err(ValueError::Expected {
                kind: "struct variant",
                value: self.0.to_static(),
            })
        }
    }
}

struct SequenceDeserializer<'de>(&'de [Value<'de>]);

impl<'de> SeqAccess<'de> for SequenceDeserializer<'de> {
    type Error = ValueError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.0.is_empty() {
            Ok(None)
        } else {
            let value = seed.deserialize(Deserializer(&self.0[0]))?;
            self.0 = &self.0[1..];
            Ok(Some(value))
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.0.len())
    }
}

struct MappingsDeserializer<'de>(&'de [(Value<'de>, Value<'de>)]);

impl<'de> MapAccess<'de> for MappingsDeserializer<'de> {
    type Error = ValueError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.0.is_empty() {
            Ok(None)
        } else {
            let key = seed.deserialize(Deserializer(&self.0[0].0))?;
            Ok(Some(key))
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(Deserializer(&self.0[0].1))?;
        self.0 = &self.0[1..];
        Ok(value)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.0.len())
    }
}

#[derive(Debug)]
pub enum Infallible {}

impl serde::ser::Error for Infallible {
    fn custom<T>(_msg: T) -> Self
    where
        T: Display,
    {
        unreachable!()
    }
}

impl std::error::Error for Infallible {}

impl Display for Infallible {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}

/// An error from deserializing a type using [`Value::deserialize_as`].
#[derive(thiserror::Error, Debug)]
pub enum ValueError {
    /// A kind of data was expected, but the [`Value`] cannot be interpreted as
    /// that kind.
    #[error("expected {kind} but got {value}")]
    Expected {
        /// The kind of data expected.
        kind: &'static str,
        /// The value that was encountered.
        value: Value<'static>,
    },
    /// A custom deserialization error. These errors originate outside of `pot`,
    #[error("{0}")]
    Custom(String),
}

impl serde::de::Error for ValueError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Custom(msg.to_string())
    }
}
