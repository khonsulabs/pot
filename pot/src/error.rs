use std::{fmt::Display, str::Utf8Error, string::FromUtf8Error};

use serde::{de, ser};

use crate::format::Kind;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // One or more variants that can be created by data structures through the
    // `ser::Error` and `de::Error` traits. For example the Serialize impl for
    // Mutex<T> might return an error because the mutex is poisoned, or the
    // Deserialize impl for a struct may return an error because a required
    // field is missing.
    #[error("{0}")]
    Message(String),
    #[error("extra data at end of input")]
    TrailingBytes,
    #[error("unexpected end of file")]
    Eof,
    #[error("value too large")]
    ImpreciseCastWouldLoseData,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("unexpected byte sequence")]
    InvalidData,
    #[error("serializing sequences of unknown size is unsupported")]
    SequenceSizeMustBeKnown,
    #[error("invalid utf8: {0}")]
    InvalidUtf8(String),
    #[error("invalid kind: {0}")]
    InvalidKind(u8),
    #[error("encountered atom kind {0:?}, expected {1:?}")]
    UnexpectedKind(Kind, Kind),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Self::InvalidUtf8(err.to_string())
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Self::InvalidUtf8(err.to_string())
    }
}
