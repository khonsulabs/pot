use std::fmt::{Debug, Display};
use std::io;
use std::str::Utf8Error;
use std::string::FromUtf8Error;

use serde::{de, ser};

use crate::format::Kind;

/// All errors that Pot may return.
#[derive(Debug)]
pub enum Error {
    /// Payload is not a Pot payload.
    NotAPot,
    /// Data was written with an incompatible version.
    IncompatibleVersion,
    /// A generic error occurred.
    Message(String),
    /// Extra data appeared at the end of the input.
    TrailingBytes,
    /// Expected more data but encountered the end of the input.
    Eof,
    /// A numerical value could not be handled without losing precision or truncation.
    ImpreciseCastWouldLoseData,
    /// An IO error occurred.
    Io(io::Error),
    /// A sequence of unknown size cannot be serialized.
    SequenceSizeMustBeKnown,
    /// String data contained invalid UTF-8 characters.
    InvalidUtf8(String),
    /// An unknown kind was encountered. Generally a sign that something else has been parsed incorrectly.
    InvalidKind(u8),
    /// Encountered an unexpected atom kind.
    UnexpectedKind(Kind, Kind),
    /// A requested symbol id was not found.
    UnknownSymbol(u64),
    /// An atom header was incorrectly formatted.
    InvalidAtomHeader,
    /// The amount of data read exceeds the configured maximum number of bytes.
    TooManyBytesRead,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotAPot => f.write_str("not a pot: invalid header"),
            Error::IncompatibleVersion => f.write_str("incompatible version"),
            Error::Message(message) => f.write_str(message),
            Error::TrailingBytes => f.write_str("extra data at end of input"),
            Error::Eof => f.write_str("unexpected end of file"),
            Error::ImpreciseCastWouldLoseData => f.write_str("numerical data cannot fit"),
            Error::Io(io) => write!(f, "io error: {io}"),
            Error::SequenceSizeMustBeKnown => {
                f.write_str("serializing sequences of unknown size is unsupported")
            }
            Error::InvalidUtf8(err) => write!(f, "invalid utf8: {err}"),
            Error::InvalidKind(kind) => write!(f, "invalid kind: {kind}"),
            Error::UnexpectedKind(encountered, expected) => write!(
                f,
                "encountered atom kind {encountered:?}, expected {expected:?}"
            ),
            Error::UnknownSymbol(sym) => write!(f, "unknown symbol {sym}"),
            Error::InvalidAtomHeader => f.write_str("an atom header was incorrectly formatted"),
            Error::TooManyBytesRead => {
                f.write_str("the deserialized value is larger than the allowed allocation limit")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
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
