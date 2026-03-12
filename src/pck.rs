use crate::ReadFromBytes;
use bitflags::bitflags;
use indexmap::IndexMap;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::borrow::Cow;
use std::io::{BufRead, IoSliceMut, Read, Seek, SeekFrom};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PckError {
    #[error("invalid magic")]
    InvalidMagic,
    #[error("invalid version")]
    InvalidVersion,
    #[error("invalid value: {0}")]
    InvalidValue(String),
    #[error("offset out of bounds")]
    OffsetOutOfBounds,
    #[error("file not found")]
    FileNotFound,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("utf8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("dds decoding error: {0}")]
    DdsDecode(#[from] dds::DecodingError),
    #[error("not implemented: {0}")]
    NotImplemented(String),
    #[error("unknown error")]
    Unknown,
}

#[derive(Debug, Clone)]
pub struct PckArchive<R> {
    reader: R,
    metadata: PckMetadata,
}

impl<R> PckArchive<R> {
    pub fn metadata(&self) -> &PckMetadata {
        &self.metadata
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.metadata.files.len()
    }
}

impl<R: Read + Seek> PckArchive<R> {
    pub fn new_from_start(mut reader: R) -> Result<Self, PckError> {
        let metadata = PckMetadata::load_from_start(&mut reader)?;
        Ok(Self { reader, metadata })
    }

    pub fn new_from_end(mut reader: R) -> Result<Self, PckError> {
        let metadata = PckMetadata::load_from_end(&mut reader)?;
        Ok(Self { reader, metadata })
    }

    pub fn new_from_offset(mut reader: R, header_offset: u64) -> Result<Self, PckError> {
        let metadata = PckMetadata::load_from_offset(&mut reader, header_offset)?;
        Ok(Self { reader, metadata })
    }

    pub fn by_path(&'_ mut self, path: impl AsRef<str>) -> Result<PckFile<'_, R>, PckError> {
        let index = self
            .metadata
            .files
            .get_index_of(path.as_ref())
            .ok_or(PckError::FileNotFound)?;
        self.by_index(index)
    }

    pub fn by_index(&'_ mut self, index: usize) -> Result<PckFile<'_, R>, PckError> {
        let (_, file_metadata) = self
            .metadata
            .files
            .get_index(index)
            .ok_or(PckError::FileNotFound)?;
        self.reader.seek(SeekFrom::Start(
            self.metadata.absolute_file_base() + file_metadata.offset,
        ))?;
        Ok(PckFile {
            file_metadata: Cow::Borrowed(file_metadata),
            reader: (&mut self.reader).take(file_metadata.size),
        })
    }
}

pub struct PckFile<'a, R> {
    file_metadata: Cow<'a, PckFileMetadata>,
    reader: std::io::Take<&'a mut R>,
}

impl<'a, R> PckFile<'a, R> {
    pub fn file_metadata(&self) -> &PckFileMetadata {
        &self.file_metadata
    }
}

impl<'a, R: Read> Read for PckFile<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
        self.reader.read_vectored(bufs)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.reader.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        self.reader.read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.reader.read_exact(buf)
    }
}

impl<'a, R: Seek> Seek for PckFile<'a, R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }

    fn rewind(&mut self) -> std::io::Result<()> {
        self.reader.rewind()
    }

    fn stream_position(&mut self) -> std::io::Result<u64> {
        self.reader.stream_position()
    }

    fn seek_relative(&mut self, offset: i64) -> std::io::Result<()> {
        self.reader.seek_relative(offset)
    }
}

impl<'a, R: BufRead> BufRead for PckFile<'a, R> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.reader.fill_buf()
    }

    fn consume(&mut self, amount: usize) {
        self.reader.consume(amount)
    }

    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.reader.read_until(byte, buf)
    }

    fn skip_until(&mut self, byte: u8) -> std::io::Result<usize> {
        self.reader.skip_until(byte)
    }

    fn read_line(&mut self, buf: &mut String) -> std::io::Result<usize> {
        self.reader.read_line(buf)
    }
}

#[derive(Debug, Clone)]
pub struct PckMetadata {
    pub header_offset: u64,
    pub header: PckHeader,
    pub files: IndexMap<Box<str>, PckFileMetadata>,
}

impl PckMetadata {
    pub fn load_from_start<R: Read + Seek>(r: &mut R) -> Result<Self, PckError> {
        Self::load_from_offset(r, 0)
    }

    pub fn load_from_end<R: Read + Seek>(r: &mut R) -> Result<Self, PckError> {
        r.seek(SeekFrom::End(-4))?;

        let magic = u32::read_ne(r)?;
        if magic != PckHeader::MAGIC {
            return Err(PckError::InvalidMagic);
        }

        let base = r.seek(SeekFrom::End(-12))?;
        let offset = u64::read_ne(r)?;
        Self::load_from_offset(r, base - offset)
    }

    pub fn load_from_offset<R: Read + Seek>(
        r: &mut R,
        header_offset: u64,
    ) -> Result<Self, PckError> {
        r.seek(SeekFrom::Start(header_offset))?;

        let magic = u32::read_ne(r)?;
        if magic != PckHeader::MAGIC {
            return Err(PckError::InvalidMagic);
        }

        let version: PckVersion = u32::read_ne(r)?
            .try_into()
            .map_err(|_| PckError::InvalidVersion)?;
        let major = u32::read_ne(r)?;
        let minor = u32::read_ne(r)?;
        let patch = u32::read_ne(r)?;
        let godot_version = GodotVersion {
            major,
            minor,
            patch,
        };

        let flags = PckArchiveFlags::from_bits_retain(u32::read_ne(r)?);
        let file_base = u64::read_ne(r)?;

        let mut salt = None;
        if version == PckVersion::V3 || version == PckVersion::V4 {
            let dir_offset = u64::read_ne(r)? + header_offset;
            if version == PckVersion::V4
                && flags.contains(PckArchiveFlags::DIR_ENCRYPTED)
                && flags.contains(PckArchiveFlags::SPARSE_BUNDLE)
            {
                let mut buffer = [0u8; 32];
                r.read_exact(&mut buffer)?;
                salt = Some(buffer);
            }
            r.seek(SeekFrom::Start(dir_offset))?;
        } else if version == PckVersion::V2 {
            // 16 x u32 reserved space
            r.seek(SeekFrom::Current(16 * 4))?;
        }

        let file_count = u32::read_ne(r)?;
        if flags.contains(PckArchiveFlags::DIR_ENCRYPTED) {
            return Err(PckError::NotImplemented("encrypted pck".into()));
        }

        let mut files = IndexMap::with_capacity(file_count as usize);
        for _ in 0..file_count {
            let path = {
                let s1 = u32::read_ne(r)?;
                let mut s = vec![0; s1 as usize];
                r.read_exact(&mut s)?;
                if let Some(end) = s.iter().position(|b| *b == b'\0') {
                    s.truncate(end);
                }
                String::from_utf8(s)?
            };

            let offset = u64::read_ne(r)?;
            let size = u64::read_ne(r)?;
            let mut md5 = [0u8; 16];
            r.read_exact(&mut md5)?;
            let flags = PckFileFlags::from_bits_retain(u32::read_ne(r)?);

            let file = PckFileMetadata {
                path: path.into_boxed_str(),
                offset,
                size,
                md5,
                flags,
            };
            files.insert(file.path.clone(), file);
        }

        Ok(Self {
            header_offset,
            header: PckHeader {
                version,
                godot_version,
                flags,
                file_base,
                salt,
            },
            files,
        })
    }

    pub(crate) fn absolute_file_base(&self) -> u64 {
        if self.header.version == PckVersion::V4
            || self.header.version == PckVersion::V3
            || (self.header.version == PckVersion::V2
                && self.header.flags.contains(PckArchiveFlags::REL_FILEBASE))
        {
            self.header.file_base + self.header_offset
        } else {
            self.header.file_base
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PckHeader {
    pub version: PckVersion,
    pub godot_version: GodotVersion,
    pub flags: PckArchiveFlags,
    pub file_base: u64,
    /// reserved space, used as salt in V4
    pub salt: Option<[u8; 32]>,
}

impl PckHeader {
    /// GDPC
    pub const MAGIC: u32 = 0x43504447;
}

#[derive(Debug, Clone)]
pub struct PckFileMetadata {
    pub path: Box<str>,
    pub offset: u64,
    pub size: u64,
    pub md5: [u8; 16],
    pub flags: PckFileFlags,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum PckVersion {
    V2 = 2,
    V3 = 3,
    V4 = 4,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GodotVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct PckArchiveFlags : u32 {
        const DIR_ENCRYPTED = 1 << 0;
        const REL_FILEBASE = 1 << 1;
        const SPARSE_BUNDLE = 1 << 2;
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct PckFileFlags : u32 {
        const ENCRYPTED = 1 << 0;
        const REMOVAL = 1 << 1;
        const DELTA = 1 << 2;
    }
}
