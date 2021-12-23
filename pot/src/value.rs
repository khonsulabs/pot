use std::{
    borrow::Cow,
    fmt::{Display, Write},
    marker::PhantomData,
};

use serde::{
    de::Visitor,
    ser::{SerializeMap, SerializeSeq},
    Deserialize, Serialize,
};

use crate::format::{Float, Integer};

/// A `Pot` encoded value. This type can be used to deserialize to and from Pot without knowing the original data structure.
#[derive(Debug, Clone, PartialEq)]
#[must_use]
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
                    write!(f, "{:02x}", byte)?;
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
    /// assert_eq!(Value::from(vec![(Value::None, Value::None)]).is_empty(), false);
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
    /// assert_eq!(Value::from(vec![(Value::None, Value::None)]).as_bool(), true);
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
    #[must_use]
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
            Value::Integer(integer) => match integer {
                Integer::I8(value) => serializer.serialize_i8(*value),
                Integer::I16(value) => serializer.serialize_i16(*value),
                Integer::I32(value) => serializer.serialize_i32(*value),
                Integer::I64(value) => serializer.serialize_i64(*value),
                Integer::I128(value) => serializer.serialize_i128(*value),
                Integer::U8(value) => serializer.serialize_u8(*value),
                Integer::U16(value) => serializer.serialize_u16(*value),
                Integer::U32(value) => serializer.serialize_u32(*value),
                Integer::U64(value) => serializer.serialize_u64(*value),
                Integer::U128(value) => serializer.serialize_u128(*value),
            },
            Value::Float(value) => match value {
                Float::F64(value) => serializer.serialize_f64(*value),
                Float::F32(value) => serializer.serialize_f32(*value),
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
        Ok(Value::Integer(Integer::I8(v)))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::I32(v)))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::I16(v)))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::I64(v)))
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::I128(v)))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::U8(v)))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::U16(v)))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::U32(v)))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::U64(v)))
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(Integer::U128(v)))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Float(Float::F32(v)))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Float(Float::F64(v)))
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
                Self::$container($container::$variant(value))
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
    assert_eq!(
        Value::from(255_f32)
            .as_integer()
            .expect("integer conversion failed")
            .as_u64()
            .unwrap(),
        255,
    );
}
