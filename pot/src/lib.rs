#![deny(unsafe_code)]
#![warn(
    // clippy::cargo,
    // missing_docs,
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

pub mod de;
mod error;
pub mod format;
pub mod reader;
pub mod ser;
pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;
use serde::{Deserialize, Serialize};

pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut output = Vec::default();
    let mut serializer = ser::Serializer::new(&mut output)?;
    value.serialize(&mut serializer)?;
    Ok(output)
}

pub fn from_slice<'a, T>(s: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = de::Deserializer::from_slice(s)?;
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.end_of_input() {
        Ok(t)
    } else {
        Err(Error::TrailingBytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

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

    fn test_serialization<S: Serialize + for<'de> Deserialize<'de> + PartialEq + Debug>(value: &S) {
        init_tracing();
        let bytes = to_vec(&value).unwrap();
        println!("{:02x?}", bytes);
        let deserialized = from_slice::<S>(&bytes).unwrap();
        assert_eq!(value, &deserialized);
    }
    use std::fmt::Debug;

    #[derive(Serialize, PartialEq, Deserialize, Debug, Default)]
    struct NumbersStruct {
        u8: u8,
        u16: u16,
        u32: u32,
        u64: u64,
        i8: i8,
        i16: i16,
        i32: i32,
        i64: i64,
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
        test_serialization(&NumbersStruct::default());
        test_serialization(&NumbersStruct {
            u8: u8::MAX,
            u16: u16::MAX,
            u32: u32::MAX,
            u64: u64::MAX,
            i8: i8::MIN,
            i16: i16::MIN,
            i32: i32::MIN,
            i64: i64::MIN,
            f32: 1.,
            f64: 1.,
        })
    }

    #[test]
    fn enums() {
        test_serialization(&EnumVariants::Unit);

        test_serialization(&EnumVariants::Tuple(0));

        test_serialization(&EnumVariants::TupleTwoArgs(1, 2));

        test_serialization(&EnumVariants::Struct { arg: 3 });
    }

    #[test]
    fn vectors() {
        test_serialization(&vec![0_u64, 1]);
        test_serialization(&vec![NumbersStruct::default(), NumbersStruct::default()]);
    }

    #[test]
    fn option() {
        test_serialization(&Option::<u64>::None);
        test_serialization(&Some(0_u64));
        test_serialization(&Some(u64::MAX));
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
}
