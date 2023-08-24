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
}

impl Default for Config {
    #[inline]
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
    /// The default allocation budget is [`usize::MAX`].
    #[inline]
    pub const fn allocation_budget(mut self, budget: usize) -> Self {
        self.allocation_budget = budget;
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
        let mut serializer = ser::Serializer::new(writer)?;
        value.serialize(&mut serializer)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::marker::PhantomData;

    use serde::{Deserializer, Serializer};

    use super::*;
    use crate::format::{Float, Integer, CURRENT_VERSION};
    use crate::value::Value;

    fn init_tracing() {
        drop(
            tracing_subscriber::fmt()
                .pretty()
                // Enable everything.
                .with_max_level(tracing::Level::TRACE)
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ENTER)
                // Set this to be the default, global collector for this application.
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
        F: FnMut(&S, &S),
    >(
        value: &S,
        check_length: Option<usize>,
        mut callback: F,
    ) {
        init_tracing();
        let bytes = to_vec(&value).unwrap();
        println!("{value:?}: {bytes:02x?}");
        let deserialized = from_slice::<S>(&bytes).unwrap();
        callback(value, &deserialized);
        if let Some(check_length) = check_length {
            // Subtract 4 bytes from the serialized output to account for the header.
            assert_eq!(bytes.len() - 4, check_length);
        }

        // Do the same, but using the reader interface.
        let mut bytes = Vec::new();
        to_writer(value, &mut bytes).unwrap();
        println!("{value:?}: {bytes:02x?}");
        let deserialized = from_reader(&bytes[..]).unwrap();
        callback(value, &deserialized);
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

        // Float packing relies on bitwise conversions and is lossless.
        test_serialization(&f64::INFINITY, Some(3));
        test_serialization(&f64::NEG_INFINITY, Some(3));
        test_serialization(&0_f64, Some(3));
        test_serialization(&-0_f64, Some(3));
        test_serialization(&0.1_f64, Some(9));
        test_serialization(&0.1_f32, Some(5));
    }

    #[test]
    fn tuples() {
        test_serialization(&(1, true, 3), None);
    }

    #[test]
    fn enums() {
        test_serialization(&EnumVariants::Unit, None);

        test_serialization(&EnumVariants::Tuple(0), None);

        test_serialization(&EnumVariants::TupleTwoArgs(1, 2), None);

        test_serialization(&EnumVariants::Struct { arg: 3 }, None);

        test_serialization(&Some(EnumVariants::Unit), None);
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

        // Test number limits.
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

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    struct TupleStruct(u32, u8);

    #[test]
    fn tuple_struct() {
        test_serialization(&TupleStruct(1, 2), None);
    }

    #[test]
    fn value() {
        macro_rules! roundtrip {
            ($value:expr) => {{
                assert_eq!(
                    from_slice::<Value<'_>>(&to_vec(&$value).unwrap()).unwrap(),
                    $value
                );
            }};
        }

        roundtrip!(Value::None);
        roundtrip!(Value::Unit);
        roundtrip!(Value::Bool(true));
        roundtrip!(Value::Bool(false));
        roundtrip!(Value::Integer(Integer::from(i8::MAX)));
        roundtrip!(Value::Integer(Integer::from(i16::MAX)));
        roundtrip!(Value::Integer(Integer::from(i32::MAX)));
        roundtrip!(Value::Integer(Integer::from(i64::MAX)));
        roundtrip!(Value::Integer(Integer::from(i128::MAX)));
        roundtrip!(Value::Integer(Integer::from(u8::MAX)));
        roundtrip!(Value::Integer(Integer::from(u16::MAX)));
        roundtrip!(Value::Integer(Integer::from(u32::MAX)));
        roundtrip!(Value::Integer(Integer::from(u64::MAX)));
        roundtrip!(Value::Integer(Integer::from(u128::MAX)));
        roundtrip!(Value::Float(Float::from(std::f64::consts::PI)));
        roundtrip!(Value::Float(Float::from(std::f32::consts::PI)));
        roundtrip!(Value::Sequence(vec![Value::None]));
        roundtrip!(Value::Mappings(vec![(Value::None, Value::Unit)]));

        let original_value = Value::Bytes(Cow::Borrowed(b"hello"));
        let encoded_bytes = to_vec(&original_value).unwrap();
        let borrowed_decoded: Value<'_> = from_slice(&encoded_bytes).unwrap();
        assert_eq!(Value::String(Cow::Borrowed("hello")), borrowed_decoded);
        assert!(matches!(borrowed_decoded, Value::String(Cow::Borrowed(_))));

        let original_value = Value::Bytes(Cow::Borrowed(b"\xFE\xED\xD0\xD0"));
        let encoded_bytes = to_vec(&original_value).unwrap();
        let borrowed_decoded: Value<'_> = from_slice(&encoded_bytes).unwrap();
        assert_eq!(
            Value::Bytes(Cow::Borrowed(b"\xFE\xED\xD0\xD0")),
            borrowed_decoded
        );
        assert!(matches!(borrowed_decoded, Value::Bytes(Cow::Borrowed(_))));
    }

    #[test]
    fn incompatible_version() {
        let mut incompatible_header = Vec::new();
        format::write_header(&mut incompatible_header, CURRENT_VERSION + 1).unwrap();
        assert!(matches!(
            from_slice::<()>(&incompatible_header),
            Err(Error::IncompatibleVersion)
        ));
    }

    #[test]
    fn invalid_char_cast() {
        let bytes = to_vec(&0x11_0000_u32).unwrap();

        assert!(matches!(
            from_slice::<char>(&bytes),
            Err(Error::InvalidUtf8(_))
        ));
    }

    #[test]
    fn bytes_to_identifier() {
        let mut valid_bytes = Vec::new();
        format::write_header(&mut valid_bytes, CURRENT_VERSION).unwrap();
        format::write_named(&mut valid_bytes).unwrap();
        format::write_bytes(&mut valid_bytes, b"Unit").unwrap();

        assert_eq!(
            from_slice::<EnumVariants>(&valid_bytes).unwrap(),
            EnumVariants::Unit
        );

        let mut invalid_bytes = Vec::new();
        format::write_header(&mut invalid_bytes, CURRENT_VERSION).unwrap();
        format::write_named(&mut invalid_bytes).unwrap();
        format::write_bytes(&mut invalid_bytes, &0xFFFF_FFFF_u32.to_be_bytes()).unwrap();

        assert!(matches!(
            from_slice::<EnumVariants>(&invalid_bytes),
            Err(Error::InvalidUtf8(_))
        ));
    }

    #[test]
    fn invalid_symbol() {
        let mut valid_bytes = Vec::new();
        format::write_header(&mut valid_bytes, CURRENT_VERSION).unwrap();
        format::write_atom_header(&mut valid_bytes, format::Kind::Symbol, 4).unwrap();
        format::write_bytes(&mut valid_bytes, &0xFFFF_FFFF_u32.to_be_bytes()).unwrap();

        assert!(matches!(
            from_slice::<Value<'_>>(&valid_bytes),
            Err(Error::InvalidUtf8(_))
        ));
    }

    #[test]
    fn unknown_special() {
        let mut invalid_bytes = Vec::new();
        format::write_header(&mut invalid_bytes, CURRENT_VERSION).unwrap();
        format::write_atom_header(
            &mut invalid_bytes,
            format::Kind::Special,
            format::SPECIAL_COUNT,
        )
        .unwrap();

        assert!(from_slice::<()>(&invalid_bytes).is_err());
    }

    /// In `BonsaiDb`, sometimes it's nice to use a `()` as an associated type
    /// as a default. To allow changing data that was previously serialized as a
    /// `()` but now has a new type, Pot allows converting between unit types
    /// and defaults of all major serialized types. The net effect is if you
    /// start with a derived `BonsaiDb` view with no `value =` argument, `()` is
    /// used instead. With this flexibility, changing the value type to another
    /// type will sometimes be able to work without requiring rebuilding the
    /// views on deployment.
    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn unit_adaptations() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(default)]
            value: u32,
        }

        let unit = to_vec(&()).unwrap();
        assert!(from_slice::<Option<()>>(&unit).unwrap().is_some());
        assert_eq!(from_slice::<Test>(&unit).unwrap().value, 0);
        assert_eq!(from_slice::<&[u8]>(&unit).unwrap(), b"");
        assert_eq!(from_slice::<serde_bytes::ByteBuf>(&unit).unwrap(), b"");
        assert_eq!(from_slice::<&str>(&unit).unwrap(), "");
        assert_eq!(from_slice::<u8>(&unit).unwrap(), 0);
        assert_eq!(from_slice::<u16>(&unit).unwrap(), 0);
        assert_eq!(from_slice::<u32>(&unit).unwrap(), 0);
        assert_eq!(from_slice::<u64>(&unit).unwrap(), 0);
        assert_eq!(from_slice::<u128>(&unit).unwrap(), 0);
        assert_eq!(from_slice::<i8>(&unit).unwrap(), 0);
        assert_eq!(from_slice::<i16>(&unit).unwrap(), 0);
        assert_eq!(from_slice::<i32>(&unit).unwrap(), 0);
        assert_eq!(from_slice::<i64>(&unit).unwrap(), 0);
        assert_eq!(from_slice::<i128>(&unit).unwrap(), 0);
        assert!(!from_slice::<bool>(&unit).unwrap());

        let none = to_vec(&Option::<()>::None).unwrap();
        assert!(from_slice::<Option<()>>(&none).unwrap().is_none());
        assert!(from_slice::<Option<Test>>(&none).unwrap().is_none());
        assert_eq!(from_slice::<&[u8]>(&none).unwrap(), b"");
        assert_eq!(from_slice::<serde_bytes::ByteBuf>(&none).unwrap(), b"");
        assert_eq!(from_slice::<&str>(&none).unwrap(), "");
        assert_eq!(from_slice::<u8>(&none).unwrap(), 0);
        assert_eq!(from_slice::<u16>(&none).unwrap(), 0);
        assert_eq!(from_slice::<u32>(&none).unwrap(), 0);
        assert_eq!(from_slice::<u64>(&none).unwrap(), 0);
        assert_eq!(from_slice::<u128>(&none).unwrap(), 0);
        assert_eq!(from_slice::<i8>(&none).unwrap(), 0);
        assert_eq!(from_slice::<i16>(&none).unwrap(), 0);
        assert_eq!(from_slice::<i32>(&none).unwrap(), 0);
        assert_eq!(from_slice::<i64>(&none).unwrap(), 0);
        assert_eq!(from_slice::<i128>(&none).unwrap(), 0);
        assert!(!from_slice::<bool>(&none).unwrap());
    }

    #[test]
    fn invalid_numbers() {
        let mut invalid_float_byte_len = Vec::new();
        format::write_header(&mut invalid_float_byte_len, CURRENT_VERSION).unwrap();
        format::write_atom_header(&mut invalid_float_byte_len, format::Kind::Float, 0).unwrap();

        assert!(from_slice::<f32>(&invalid_float_byte_len).is_err());

        assert!(format::Float::read_from(
            format::Kind::Symbol,
            0,
            &mut &invalid_float_byte_len[..]
        )
        .is_err(),);

        let mut invalid_signed_byte_len = Vec::new();
        format::write_header(&mut invalid_signed_byte_len, CURRENT_VERSION).unwrap();
        format::write_atom_header(&mut invalid_signed_byte_len, format::Kind::Int, 10).unwrap();

        assert!(from_slice::<i32>(&invalid_signed_byte_len).is_err());

        assert!(format::Integer::read_from(
            format::Kind::Symbol,
            0,
            &mut &invalid_signed_byte_len[..]
        )
        .is_err(),);

        let mut invalid_unsigned_byte_len = Vec::new();
        format::write_header(&mut invalid_unsigned_byte_len, CURRENT_VERSION).unwrap();
        format::write_atom_header(&mut invalid_unsigned_byte_len, format::Kind::UInt, 10).unwrap();

        assert!(from_slice::<u32>(&invalid_unsigned_byte_len).is_err());
    }

    #[test]
    #[allow(clippy::unnecessary_mut_passed)] // It's necessary.
    fn not_human_readable() {
        let mut bytes = Vec::new();
        let mut serializer = ser::Serializer::new(&mut bytes).unwrap();
        assert!(!(&mut serializer).is_human_readable());
        ().serialize(&mut serializer).unwrap();

        let bytes = to_vec(&()).unwrap();
        let mut deserializer = de::Deserializer::from_slice(&bytes, usize::MAX).unwrap();
        assert!(!(&mut deserializer).is_human_readable());
    }

    #[test]
    fn unexpected_eof() {
        let mut invalid_bytes = Vec::new();
        format::write_header(&mut invalid_bytes, CURRENT_VERSION).unwrap();
        format::write_atom_header(&mut invalid_bytes, format::Kind::Bytes, 10).unwrap();
        assert!(matches!(
            from_slice::<Vec<u8>>(&invalid_bytes),
            Err(Error::Eof)
        ));
    }

    #[test]
    fn too_big_read() {
        let mut invalid_bytes = Vec::new();
        format::write_header(&mut invalid_bytes, CURRENT_VERSION).unwrap();
        format::write_atom_header(&mut invalid_bytes, format::Kind::Bytes, 10).unwrap();
        assert!(matches!(
            Config::default()
                .allocation_budget(9)
                .deserialize::<Vec<u8>>(&invalid_bytes),
            Err(Error::TooManyBytesRead)
        ));
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Flatten {
        #[serde(flatten)]
        structure: Flattened,
        #[serde(flatten)]
        enumeration: EnumVariants,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Flattened {
        field: String,
    }

    #[test]
    fn test_flatten() {
        test_serialization(
            &Flatten {
                structure: Flattened {
                    field: String::from("flat"),
                },
                enumeration: EnumVariants::Struct { arg: 1 },
            },
            None,
        );
    }

    #[test]
    fn direct_value_serialization() {
        fn roundtrip<T: Serialize + for<'de> Deserialize<'de> + PartialEq + Debug>(value: &T) {
            let as_value = Value::from_serialize(value).unwrap();
            let deserialized = as_value.deserialize_as::<T>().unwrap();
            assert_eq!(&deserialized, value);
        }

        roundtrip(&NumbersStruct {
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
        });

        roundtrip(&EnumVariants::Struct { arg: 1 });
        roundtrip(&EnumVariants::Tuple(1));
        roundtrip(&EnumVariants::TupleTwoArgs(1, 2));
        roundtrip(&EnumVariants::Unit);
        roundtrip(&Some(1_u32));
        roundtrip(&"hello".to_string());
        roundtrip(&b"hello".to_vec());
    }

    #[test]
    fn borrowed_value_serialization() {
        #[track_caller]
        fn check<T, U>(value: &T)
        where
            T: Serialize + Debug,
            U: Debug + PartialEq<T> + for<'de> Deserialize<'de>,
        {
            let as_value = Value::from_serialize(value).unwrap();
            let deserialized = as_value.deserialize_as::<U>().unwrap();
            assert_eq!(&deserialized, value);
        }

        check::<_, Vec<u8>>(&b"hello");
        check::<_, String>(&"hello");
    }

    #[test]
    fn value_error() {
        #[derive(Debug)]
        struct Fallible;

        impl Serialize for Fallible {
            fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                Err(serde::ser::Error::custom("oh no!"))
            }
        }

        assert_eq!(
            Value::from_serialize(Fallible),
            Err(ValueError::Custom(String::from("oh no!")))
        );
    }

    #[test]
    fn persistent_symbols_slice() {
        let mut sender = ser::SymbolMap::default();
        let mut receiver = de::SymbolList::default();

        let mut bytes = Vec::new();
        NumbersStruct::default()
            .serialize(&mut sender.serializer_for(&mut bytes).unwrap())
            .unwrap();
        let _result =
            NumbersStruct::deserialize(&mut receiver.deserializer_for_slice(&bytes).unwrap())
                .unwrap();
        let symbol_count_after_first_send = receiver.len();
        let first_payload_len = bytes.len();

        // Send again, confirm the symbol list didn't grow.
        bytes.clear();
        NumbersStruct::default()
            .serialize(&mut sender.serializer_for(&mut bytes).unwrap())
            .unwrap();
        let _result =
            NumbersStruct::deserialize(&mut receiver.deserializer_for_slice(&bytes).unwrap())
                .unwrap();
        assert_eq!(symbol_count_after_first_send, receiver.len());
        println!(
            "First: {first_payload_len} bytes; Second: {} bytes",
            bytes.len()
        );
        assert!(first_payload_len > bytes.len());
    }

    #[test]
    fn persistent_symbols_read() {
        let mut sender = ser::SymbolMap::default();
        let mut receiver = de::SymbolList::default();

        let mut bytes = Vec::new();
        NumbersStruct::default()
            .serialize(&mut sender.serializer_for(&mut bytes).unwrap())
            .unwrap();
        let _result =
            NumbersStruct::deserialize(&mut receiver.deserializer_for(&bytes[..]).unwrap())
                .unwrap();
        let symbol_count_after_first_send = receiver.len();
        let first_payload_len = bytes.len();

        // Send again, confirm the symbol list didn't grow.
        bytes.clear();
        NumbersStruct::default()
            .serialize(&mut sender.serializer_for(&mut bytes).unwrap())
            .unwrap();
        let _result =
            NumbersStruct::deserialize(&mut receiver.deserializer_for(&bytes[..]).unwrap())
                .unwrap();
        assert_eq!(symbol_count_after_first_send, receiver.len());
        println!(
            "First: {first_payload_len} bytes; Second: {} bytes",
            bytes.len()
        );
        assert!(first_payload_len > bytes.len());
    }

    #[test]
    fn symbol_map_serialization() {
        #[derive(Serialize, Deserialize, Default, Eq, PartialEq, Debug)]
        struct Payload {
            a: usize,
            b: usize,
        }

        let mut sender = crate::ser::SymbolMap::default();
        assert!(sender.is_empty());
        let mut receiver = crate::de::SymbolMap::new();
        assert!(receiver.is_empty());
        let mut bytes = Vec::new();

        // Send the first payload, populating the map.
        Payload::default()
            .serialize(&mut sender.serializer_for(&mut bytes).unwrap())
            .unwrap();
        assert_eq!(sender.len(), 2);

        assert_eq!(
            Payload::deserialize(&mut receiver.deserializer_for_slice(&bytes).unwrap()).unwrap(),
            Payload::default()
        );
        assert_eq!(receiver.len(), 2);

        // Serialize the maps.
        let serialized_sender = crate::to_vec(&sender).unwrap();
        let serialized_receiver = crate::to_vec(&receiver).unwrap();
        // The serialization formats are the same despite using different
        // in-memory representations. This allows pre-serializing a dictionary
        // before starting the intial payload.
        assert_eq!(serialized_sender, serialized_receiver);
        let mut deserialized_sender =
            crate::from_slice::<crate::ser::SymbolMap>(&serialized_sender).unwrap();
        let mut deserialized_receiver =
            crate::from_slice::<crate::de::SymbolMap>(&serialized_receiver).unwrap();

        // Create a new payload and serialize it. Ensure the payloads produced
        // by the serialized map and the original map are identical.
        let new_payload = Payload { a: 1, b: 2 };
        bytes.clear();
        new_payload
            .serialize(&mut sender.serializer_for(&mut bytes).unwrap())
            .unwrap();
        let mut from_serialized_sender = Vec::new();
        new_payload
            .serialize(
                &mut deserialized_sender
                    .serializer_for(&mut from_serialized_sender)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(bytes, from_serialized_sender);

        // Deserialize the payload
        assert_eq!(
            Payload::deserialize(&mut receiver.deserializer_for_slice(&bytes).unwrap()).unwrap(),
            new_payload
        );
        assert_eq!(
            Payload::deserialize(
                &mut deserialized_receiver
                    .deserializer_for_slice(&bytes)
                    .unwrap()
            )
            .unwrap(),
            new_payload
        );
    }

    #[test]
    fn symbol_map_population() {
        let mut map = crate::ser::SymbolMap::default();
        map.populate_from(&NumbersStruct::default()).unwrap();
        map.populate_from(&EnumVariants::Struct { arg: 1 }).unwrap();
        map.populate_from(&EnumVariants::Tuple(0)).unwrap();
        map.populate_from(&EnumVariants::TupleTwoArgs(0, 1))
            .unwrap();
        assert_eq!(map.populate_from(&EnumVariants::Unit).unwrap(), 1);
        assert_eq!(map.populate_from(&EnumVariants::Unit).unwrap(), 0);
        dbg!(map);
    }
}
