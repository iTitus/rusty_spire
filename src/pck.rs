use crate::ReadFromBytes;
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
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

#[derive(Debug)]
pub struct PckReader<R> {
    pub pck: Pck,
    reader: R,
    index: HashMap<String, usize>,
}

impl<R: Read + Seek> PckReader<R> {
    fn new(reader: R, pck: Pck) -> Result<PckReader<R>, PckError> {
        let mut index = HashMap::default();
        for (idx, f) in pck.files.iter().enumerate() {
            index.insert(f.path.clone(), idx);
        }

        Ok(Self { pck, reader, index })
    }

    pub fn new_from_start(mut reader: R) -> Result<Self, PckError> {
        let pck = Pck::load_from_start(&mut reader)?;
        Self::new(reader, pck)
    }

    pub fn new_from_end(mut reader: R) -> Result<Self, PckError> {
        let pck = Pck::load_from_end(&mut reader)?;
        Self::new(reader, pck)
    }

    pub fn new_from_offset(mut reader: R, header_offset: u64) -> Result<Self, PckError> {
        let pck = Pck::load_from_offset(&mut reader, header_offset)?;
        Self::new(reader, pck)
    }

    pub fn read(&mut self, path: impl AsRef<str>) -> Result<Vec<u8>, PckError> {
        let idx = self.index[path.as_ref()];
        let f = &self.pck.files[idx];
        self.reader
            .seek(SeekFrom::Start(self.pck.absolute_file_base() + f.offset))?;
        let mut buf = vec![0; f.size as usize];
        self.reader.read_exact(&mut buf)?;
        Ok(buf)
    }
}

#[derive(Debug, Clone)]
pub struct Pck {
    pub header_offset: u64,
    pub header: PckHeader,
    pub files: Vec<PckFile>,
}

impl Pck {
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

        let version = u32::read_ne(r)?;
        if !matches!(version, 2..=4) {
            return Err(PckError::InvalidVersion);
        }

        let major = u32::read_ne(r)?;
        let minor = u32::read_ne(r)?;
        let patch = u32::read_ne(r)?;
        let godot_version = GodotVersion {
            major,
            minor,
            patch,
        };

        let flags = u32::read_ne(r)?;
        let file_base = u64::read_ne(r)?;

        let actual_file_base = if version == 4
            || version == 3
            || (version == 2 && (flags & PckHeader::FLAG_REL_FILEBASE) != 0)
        {
            file_base + header_offset
        } else {
            file_base
        };

        let mut salt = None;
        if version == 4 || version == 3 {
            let dir_offset_ = u64::read_ne(r)?;
            if version == 4
                && (flags & PckHeader::FLAG_DIR_ENCRYPTED) != 0
                && (flags & PckHeader::FLAG_SPARSE_BUNDLE) != 0
            {
                let mut buffer = [0u8; 32];
                r.read_exact(&mut buffer)?;
                salt = Some(buffer);
            }
            r.seek(SeekFrom::Start(header_offset + dir_offset_))?;
        } else if version == 2 {
            // 16 x u32 reserved space
            r.seek(SeekFrom::Current(16 * 4))?;
        }

        let file_count = u32::read_ne(r)?;
        if (flags & PckHeader::FLAG_DIR_ENCRYPTED) != 0 {
            unimplemented!();
        }

        let mut files = vec![];
        for _ in 0..file_count {
            let path = {
                let s1 = u32::read_ne(r)?;
                let mut s = vec![0; s1 as usize];
                r.read_exact(&mut s)?;
                let end = s.iter().position(|b| *b == b'\0').unwrap_or(s.len());
                s.truncate(end);
                String::from_utf8(s)?
            };

            let offset = u64::read_ne(r)?;
            let _actual_offset = actual_file_base + offset;

            let size = u64::read_ne(r)?;
            let mut md5 = [0u8; 16];
            r.read_exact(&mut md5)?;
            let flags = u32::read_ne(r)?;

            files.push(PckFile {
                path,
                offset,
                size,
                md5,
                flags,
            })
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
        if self.header.version == 4
            || self.header.version == 3
            || (self.header.version == 2 && (self.header.flags & PckHeader::FLAG_REL_FILEBASE) != 0)
        {
            self.header.file_base + self.header_offset
        } else {
            self.header.file_base
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PckHeader {
    pub version: u32,
    pub godot_version: GodotVersion,
    pub flags: u32,
    pub file_base: u64,
    /// reserved space, used as salt in V4
    pub salt: Option<[u8; 32]>,
}

impl PckHeader {
    /// GDPC
    pub const MAGIC: u32 = 0x43504447;
    pub const FLAG_DIR_ENCRYPTED: u32 = 0x1;
    pub const FLAG_REL_FILEBASE: u32 = 0x2;
    pub const FLAG_SPARSE_BUNDLE: u32 = 0x4;
    pub const FLAG_FILE_ENCRYPTED: u32 = 0x1;
    pub const FLAG_FILE_REMOVAL: u32 = 0x2;
    pub const FLAG_FILE_DELTA: u32 = 0x4;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GodotVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Debug, Clone)]
pub struct PckFile {
    pub path: String,
    pub offset: u64,
    pub size: u64,
    pub md5: [u8; 16],
    pub flags: u32,
}
