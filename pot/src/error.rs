use std::{fmt::Display, str::Utf8Error, string::FromUtf8Error};

use serde::{de, ser};

use crate::format::Kind;

/// All errors that Pot may return.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Payload is not a Pot payload.
    #[error("not a pot: invalid header")]
    NotAPot,
    /// Data was written with an incompatible version.
    #[error("incompatible version")]
    IncompatibleVersion,
    /// A generic error occurred.
    #[error("{0}")]
    Message(String),
    /// Extra data appeared at the end of the input.
    #[error("extra data at end of input")]
    TrailingBytes,
    /// Expected more data but encountered the end of the input.
    #[error("unexpected end of file")]
    Eof,
    /// A numerical value could not be handled without losing precision or truncation.
    #[error("numerical data cannot fit")]
    ImpreciseCastWouldLoseData,
    /// An error occurred from io.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// A sequence of unknown size cannot be serialized.
    #[error("serializing sequences of unknown size is unsupported")]
    SequenceSizeMustBeKnown,
    /// String data contained invalid utf-8 characters.
    #[error("invalid utf8: {0}")]
    InvalidUtf8(String),
    /// An unknown kind was encountered. Generally a sign that something else has been parsed incorrectly.
    #[error("invalid kind: {0}")]
    InvalidKind(u8),
    /// Encountered an unexpected atom kind.
    #[error("encountered atom kind {0:?}, expected {1:?}")]
    UnexpectedKind(Kind, Kind),
    /// A requested symbol id was not found.
    #[error("unknown symbol {0}")]
    UnknownSymbol(u64),
    /// An atom header was incorrectly formatted.
    #[error("an atom header was incorrectly formatted")]
    InvalidAtomHeader,
    /// The amount of data read exceeds the configured maximum number of bytes.
    #[error("the deserialized value is larger than the allowed allocation limit")]
    TooManyBytesRead,
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
