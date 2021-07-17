use std::{fmt::Debug, io::Read};

use byteorder::ReadBytesExt;

use crate::Error;

/// A reader that can temporarily buffer bytes read.
pub trait Reader<'de>: ReadBytesExt + Debug {
    /// Read exactly `length` bytes and return a reference to the buffer.
    fn buffered_read_bytes(&mut self, length: usize) -> Result<&'de [u8], Error>;
}

/// Reads data from a slice.
#[allow(clippy::module_name_repetitions)]
pub struct SliceReader<'a> {
    pub(crate) data: &'a [u8],
}

impl<'a> Debug for SliceReader<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SliceReader")
            .field(
                "preview",
                &format!("{:0x?}", &self.data[..8.min(self.data.len())]),
            )
            .finish()
    }
}

impl<'a> From<&'a [u8]> for SliceReader<'a> {
    fn from(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'de> Reader<'de> for SliceReader<'de> {
    fn buffered_read_bytes(&mut self, length: usize) -> Result<&'de [u8], Error> {
        if length > self.data.len() {
            self.data = &self.data[self.data.len()..];
            Err(Error::Eof)
        } else {
            let (start, remaining) = self.data.split_at(length);
            self.data = remaining;
            Ok(start)
        }
    }
}

impl<'a> Read for SliceReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let remaining_length = self.data.len();
        let (to_copy, remaining) = self.data.split_at(remaining_length.min(buf.len()));
        buf[..to_copy.len()].copy_from_slice(to_copy);
        self.data = remaining;
        Ok(to_copy.len())
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.read(buf).map(|_| ())
    }
}

/// A reader over [`ReadBytesExt`].
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct IoReader<R: ReadBytesExt + Debug> {
    pub(crate) reader: R,
    buffer: Vec<u8>,
}
