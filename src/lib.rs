use crate::pck::PckError;
use std::io::Read;

pub mod pck;
pub mod save;
mod serde;
pub mod texture;

#[allow(unused)]
pub(crate) trait ReadFromBytes: Sized {
    fn read_ne<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PckError>;
    fn read_be<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PckError>;
    fn read_le<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PckError>;
}

impl ReadFromBytes for u64 {
    fn read_ne<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PckError> {
        let mut buf = [0u8; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_ne_bytes(buf))
    }

    fn read_be<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PckError> {
        let mut buf = [0u8; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_be_bytes(buf))
    }

    fn read_le<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PckError> {
        let mut buf = [0u8; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }
}

impl ReadFromBytes for u32 {
    fn read_ne<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PckError> {
        let mut buf = [0u8; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_ne_bytes(buf))
    }

    fn read_be<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PckError> {
        let mut buf = [0u8; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_be_bytes(buf))
    }

    fn read_le<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PckError> {
        let mut buf = [0u8; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }
}

impl ReadFromBytes for u16 {
    fn read_ne<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PckError> {
        let mut buf = [0u8; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_ne_bytes(buf))
    }

    fn read_be<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PckError> {
        let mut buf = [0u8; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_be_bytes(buf))
    }

    fn read_le<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PckError> {
        let mut buf = [0u8; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }
}

impl ReadFromBytes for u8 {
    fn read_ne<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PckError> {
        let mut buf = [0u8; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_ne_bytes(buf))
    }

    fn read_be<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PckError> {
        let mut buf = [0u8; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_be_bytes(buf))
    }

    fn read_le<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PckError> {
        let mut buf = [0u8; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }
}
