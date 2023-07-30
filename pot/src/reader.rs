use std::fmt::Debug;
use std::io::Read;

use byteorder::ReadBytesExt;

use crate::Error;

/// A reader that can temporarily buffer bytes read.
pub trait Reader<'de>: ReadBytesExt {
    /// Reads exactly `length` bytes.
    ///
    /// If the reader supports borrowing bytes, [`BufferedBytes::Data`] should
    /// be returned. Otherwise, the bytes will be read into `scratch`. `scratch`
    /// should only be assumed to be valid if [`BufferedBytes::Scratch`] is
    /// returned.
    fn buffered_read_bytes(
        &mut self,
        length: usize,
        scratch: &mut Vec<u8>,
    ) -> Result<BufferedBytes<'de>, Error>;
}

/// Bytes that have been read into a buffer.
#[derive(Debug)]
pub enum BufferedBytes<'de> {
    /// The bytes that have been read can be borrowed from the source.
    Data(&'de [u8]),
    /// The bytes that have been read have been stored in the scratch buffer
    /// passed to the function reading bytes.
    Scratch,
}

impl BufferedBytes<'_> {
    /// Resolves the bytes to a byte slice.
    #[inline]
    #[must_use]
    pub fn as_slice<'a>(&'a self, scratch: &'a [u8]) -> &'a [u8] {
        match self {
            BufferedBytes::Data(data) => data,
            BufferedBytes::Scratch => scratch,
        }
    }
}

/// Reads data from a slice.
#[allow(clippy::module_name_repetitions)]
pub struct SliceReader<'a> {
    pub(crate) data: &'a [u8],
}

impl<'a> SliceReader<'a> {
    /// Returns the remaining bytes to read.
    #[must_use]
    #[inline]
    pub const fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if there are no bytes remaining to read.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
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
    #[inline]
    fn from(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> From<SliceReader<'a>> for &'a [u8] {
    #[inline]
    fn from(reader: SliceReader<'a>) -> Self {
        reader.data
    }
}

impl<'de> Reader<'de> for SliceReader<'de> {
    #[inline]
    fn buffered_read_bytes(
        &mut self,
        length: usize,
        _scratch: &mut Vec<u8>,
    ) -> Result<BufferedBytes<'de>, Error> {
        if length > self.data.len() {
            self.data = &self.data[self.data.len()..];
            Err(Error::Eof)
        } else {
            let (start, remaining) = self.data.split_at(length);
            self.data = remaining;
            Ok(BufferedBytes::Data(start))
        }
    }
}

impl<'a> Read for SliceReader<'a> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let remaining_length = self.data.len();
        let (to_copy, remaining) = self.data.split_at(remaining_length.min(buf.len()));
        buf[..to_copy.len()].copy_from_slice(to_copy);
        self.data = remaining;
        Ok(to_copy.len())
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.read(buf).map(|_| ())
    }
}

/// A reader over [`ReadBytesExt`].
#[allow(clippy::module_name_repetitions)]
pub struct IoReader<R: ReadBytesExt> {
    pub(crate) reader: R,
}
impl<R: ReadBytesExt> IoReader<R> {
    pub(crate) const fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<'de, R: ReadBytesExt> Reader<'de> for IoReader<R> {
    #[inline]
    fn buffered_read_bytes(
        &mut self,
        length: usize,
        scratch: &mut Vec<u8>,
    ) -> Result<BufferedBytes<'de>, Error> {
        scratch.resize(length, 0);
        self.reader.read_exact(scratch)?;
        Ok(BufferedBytes::Scratch)
    }
}

impl<R: ReadBytesExt> Read for IoReader<R> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        self.reader.read_vectored(bufs)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.reader.read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        self.reader.read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.reader.read_exact(buf)
    }
}

#[test]
fn slice_reader_pub_methods() {
    let mut reader = SliceReader::from(&b"a"[..]);
    assert_eq!(reader.len(), 1);
    assert!(!reader.is_empty());
    reader.read_exact(&mut [0]).unwrap();

    assert_eq!(reader.len(), 0);
    assert!(reader.is_empty());
    assert_eq!(<&[u8]>::from(reader), b"");
}
