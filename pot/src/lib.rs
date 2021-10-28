//! A concise serialization format written for `BonsaiDb`.

#![forbid(unsafe_code)]
#![warn(
    clippy::cargo,
    missing_docs,
    // clippy::missing_docs_in_private_items,
    clippy::nursery,
    clippy::pedantic,
    future_incompatible,
    rust_2018_idioms,
)]
#![cfg_attr(doc, deny(rustdoc::all))]
#![allow(
    clippy::missing_errors_doc, // TODO clippy::missing_errors_doc
    clippy::option_if_let_else,
    clippy::used_underscore_binding, // false positive with tracing
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
pub use error::Error;
/// A result alias that returns [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
use serde::{Deserialize, Serialize};

/// Serialize `value` into a pot.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    Config::default().serialize(value)
}

/// Restore a previously serialized value from a pot.
pub fn from_slice<'a, T>(serialized: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    Config::default().deserialize(serialized)
}

/// Serialization and deserialization configuration.
#[must_use]
pub struct Config {
    allocation_budget: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            allocation_budget: usize::MAX,
        }
    }
}

impl Config {
    /// Sets the maximum number of bytes able to be allocated. This is not
    /// guaranteed to be perfectly accurate, due to the limitations of serde
    /// deserializers. Pot can keep track of how many bytes it thinks its
    /// allocating, but a deserializer can always allocate more memory than Pot
    /// can be aware of.
    ///
    /// The default allocation budget is `usize::MAX`.
    pub const fn allocation_budget(mut self, budget: usize) -> Self {
        self.allocation_budget = budget;
        self
    }

    /// Deserializes a value from a slice using the configured options.
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

    /// Serializes a value to a `Vec` using the configured options.
    #[allow(clippy::unused_self)]
    pub fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>> {
        let mut output = Vec::default();
        let mut serializer = ser::Serializer::new(&mut output)?;
        value.serialize(&mut serializer)?;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, marker::PhantomData};

    use serde_json::{value::Value, Number};

    use super::*;

    fn init_tracing() {
        drop(
            tracing_subscriber::fmt()
                .pretty()
                // enable everything
                .with_max_level(tracing::Level::TRACE)
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ENTER)
                // sets this to be the default, global collector for this application.
                .try_init(),
        );
    }

    fn test_serialization<S: Serialize + for<'de> Deserialize<'de> + PartialEq + Debug>(
        value: &S,
        check_length: Option<usize>,
    ) {
        test_serialization_with(value, check_length, |value, deserialized| {
            assert_eq!(value, deserialized);
        });
    }

    fn test_serialization_with<
        S: Serialize + for<'de> Deserialize<'de> + PartialEq + Debug,
        F: FnOnce(&S, &S),
    >(
        value: &S,
        check_length: Option<usize>,
        callback: F,
    ) {
        init_tracing();
        let bytes = to_vec(&value).unwrap();
        println!("{:?}: {:02x?}", value, bytes);
        let deserialized = from_slice::<S>(&bytes).unwrap();
        callback(value, &deserialized);
        if let Some(check_length) = check_length {
            // Subtract 4 bytes from the serialized output to account for the header.
            assert_eq!(bytes.len() - 4, check_length);
        }
    }

    use std::fmt::Debug;

    #[derive(Serialize, PartialEq, Deserialize, Debug, Default)]
    struct NumbersStruct {
        u8: u8,
        u16: u16,
        char: char,
        u32: u32,
        u64: u64,
        u128: u128,
        i8: i8,
        i16: i16,
        i32: i32,
        i64: i64,
        i128: i128,
        f32: f32,
        f64: f64,
    }

    #[derive(Serialize, PartialEq, Deserialize, Debug)]
    enum EnumVariants {
        Unit,
        Tuple(u64),
        TupleTwoArgs(u64, u64),
        Struct { arg: u64 },
    }

    #[test]
    fn numbers() {
        test_serialization(&NumbersStruct::default(), None);
        test_serialization(
            &NumbersStruct {
                u8: u8::MAX,
                u16: u16::MAX,
                char: char::MAX,
                u32: u32::MAX,
                u64: u64::MAX,
                u128: u128::MAX,
                i8: i8::MIN,
                i16: i16::MIN,
                i32: i32::MIN,
                i64: i64::MIN,
                i128: i128::MIN,
                f32: 1.,
                f64: 1.,
            },
            None,
        );
    }

    #[test]
    fn number_packing() {
        test_serialization(&0_u128, Some(2));
        test_serialization(&(2_u128.pow(8) - 1), Some(2));
        test_serialization(&2_u128.pow(8), Some(3));
        test_serialization(&(2_u128.pow(16) - 1), Some(3));
        test_serialization(&2_u128.pow(16), Some(4));
        test_serialization(&(2_u128.pow(24) - 1), Some(4));
        test_serialization(&2_u128.pow(24), Some(5));
        test_serialization(&(2_u128.pow(32) - 1), Some(5));
        test_serialization(&2_u128.pow(32), Some(7));
        test_serialization(&(2_u128.pow(48) - 1), Some(7));
        test_serialization(&2_u128.pow(48), Some(9));
        test_serialization(&(2_u128.pow(64) - 1), Some(9));
        test_serialization(&2_u128.pow(64), Some(17));

        test_serialization(&0_i128, Some(2));
        test_serialization(&(2_i128.pow(7) - 1), Some(2));
        test_serialization(&2_i128.pow(7), Some(3));
        test_serialization(&(2_i128.pow(15) - 1), Some(3));
        test_serialization(&2_i128.pow(15), Some(4));
        test_serialization(&(2_i128.pow(23) - 1), Some(4));
        test_serialization(&2_i128.pow(23), Some(5));
        test_serialization(&(2_i128.pow(31) - 1), Some(5));
        test_serialization(&2_i128.pow(31), Some(7));
        test_serialization(&(2_i128.pow(47) - 1), Some(7));
        test_serialization(&2_i128.pow(47), Some(9));
        test_serialization(&-(2_i128.pow(7)), Some(2));
        test_serialization(&-(2_i128.pow(7) + 1), Some(3));
        test_serialization(&-(2_i128.pow(15)), Some(3));
        test_serialization(&-(2_i128.pow(15) + 1), Some(4));
        test_serialization(&-(2_i128.pow(23)), Some(4));
        test_serialization(&-(2_i128.pow(23) + 1), Some(5));
        test_serialization(&-(2_i128.pow(31)), Some(5));
        test_serialization(&-(2_i128.pow(31) + 1), Some(7));
        test_serialization(&-(2_i128.pow(47)), Some(7));
        test_serialization(&-(2_i128.pow(47) + 1), Some(9));
        test_serialization(&-(2_i128.pow(63)), Some(9));
        test_serialization(&-(2_i128.pow(63) + 1), Some(17));

        // Float packing relies on bitwise conversions and are lossless.
        test_serialization(&f64::INFINITY, Some(3));
        test_serialization(&f64::NEG_INFINITY, Some(3));
        test_serialization(&0_f64, Some(3));
        test_serialization(&-0_f64, Some(3));
        test_serialization(&0.1_f64, Some(9));
        test_serialization(&0.1_f32, Some(5));
    }

    #[test]
    fn enums() {
        test_serialization(&EnumVariants::Unit, None);

        test_serialization(&EnumVariants::Tuple(0), None);

        test_serialization(&EnumVariants::TupleTwoArgs(1, 2), None);

        test_serialization(&EnumVariants::Struct { arg: 3 }, None);
    }

    #[test]
    fn vectors() {
        test_serialization(&vec![0_u64, 1], None);
        test_serialization(
            &vec![NumbersStruct::default(), NumbersStruct::default()],
            None,
        );
    }

    #[test]
    fn option() {
        test_serialization(&Option::<u64>::None, None);
        test_serialization(&Some(0_u64), None);
        test_serialization(&Some(u64::MAX), None);
    }

    #[test]
    fn phantom() {
        test_serialization(&PhantomData::<u64>, None);
    }

    #[derive(Serialize, PartialEq, Deserialize, Debug, Default)]
    struct StringsAndBytes<'a> {
        bytes: Cow<'a, [u8]>,
        #[serde(with = "serde_bytes")]
        bytes_borrowed: Cow<'a, [u8]>,
        #[serde(with = "serde_bytes")]
        serde_bytes_byte_slice: &'a [u8],
        #[serde(with = "serde_bytes")]
        serde_bytes_byte_vec: Vec<u8>,
        str_ref: &'a str,
        string: String,
    }

    #[test]
    fn borrowing_data() {
        let original = StringsAndBytes {
            bytes: Cow::Borrowed(b"hello"),
            bytes_borrowed: Cow::Borrowed(b"hello"),
            serde_bytes_byte_slice: b"hello",
            serde_bytes_byte_vec: b"world".to_vec(),
            str_ref: "hello",
            string: String::from("world"),
        };
        let serialized = to_vec(&original).unwrap();
        let deserialized = from_slice(&serialized).unwrap();
        assert_eq!(original, deserialized);
        assert!(matches!(deserialized.bytes_borrowed, Cow::Borrowed(_)));
    }

    #[test]
    fn limiting_input() {
        let original = StringsAndBytes {
            bytes: Cow::Borrowed(b"hello"),
            bytes_borrowed: Cow::Borrowed(b"hello"),
            serde_bytes_byte_slice: b"hello",
            serde_bytes_byte_vec: b"world".to_vec(),
            str_ref: "hello",
            string: String::from("world"),
        };
        let serialized = to_vec(&original).unwrap();
        // There are 6 values that contain 5 bytes each. A limit of 30 should be perfect.
        assert!(Config::default()
            .allocation_budget(30)
            .deserialize::<StringsAndBytes<'_>>(&serialized)
            .is_ok());
        assert!(Config::default()
            .allocation_budget(29)
            .deserialize::<StringsAndBytes<'_>>(&serialized)
            .is_err());

        // Test number limits
        let serialized = to_vec(&NumbersStruct {
            u8: u8::MAX,
            u16: u16::MAX,
            char: char::MAX,
            u32: u32::MAX,
            u64: u64::MAX,
            u128: u128::MAX,
            i8: i8::MIN,
            i16: i16::MIN,
            i32: i32::MIN,
            i64: i64::MIN,
            i128: i128::MIN,
            f32: f32::MAX,
            f64: f64::MIN,
        })
        .unwrap();
        assert!(Config::default()
            .allocation_budget(78)
            .deserialize::<NumbersStruct>(&serialized)
            .is_ok());
        assert!(Config::default()
            .allocation_budget(77)
            .deserialize::<NumbersStruct>(&serialized)
            .is_err());
    }

    #[test]
    fn deserialize_any() {
        test_serialization(&Value::Null, None);
        test_serialization(&Value::Bool(false), None);
        test_serialization(&Value::Bool(true), None);
        test_serialization(&Value::Array(vec![serde_json::value::Value::Null]), None);
        test_serialization(&Value::Number(Number::from_f64(1.).unwrap()), None);
        test_serialization(&Value::String(String::from("Hello world")), None);
        test_serialization(
            &Value::Object(
                [(String::from("key"), Value::Bool(true))]
                    .into_iter()
                    .collect(),
            ),
            None,
        );
    }
}