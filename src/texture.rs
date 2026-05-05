use crate::ReadFromBytes;
use crate::pck::PckError;
use bitflags::bitflags;
use dds::{ColorFormat, ImageViewMut};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::io::{BufRead, Cursor, Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct CompressedTexture2d {
    pub header: CompressedTexture2dHeader,
    pub images: Vec<image::DynamicImage>,
}

impl CompressedTexture2d {
    /// GST2
    const MAGIC: u32 = 0x47535432;

    pub fn load<R: BufRead + Seek + ?Sized>(r: &mut R) -> Result<Self, PckError> {
        let magic = u32::read_be(r)?;
        if magic != Self::MAGIC {
            return Err(PckError::InvalidMagic);
        }

        let version: CompressedTexture2dVersion = u32::read_ne(r)?
            .try_into()
            .map_err(|_| PckError::InvalidVersion)?;

        let width = u32::read_ne(r)?;
        let height = u32::read_ne(r)?;
        let format_flags = CompressedTexture2dDataFormatFlags::from_bits_retain(u32::read_ne(r)?);

        let mipmap_limit = u32::read_ne(r)?;
        // skip reserved
        for _ in 0..3 {
            let _ = u32::read_ne(r)?;
        }

        let data_format: CompressedTexture2dDataFormat = u32::read_ne(r)?
            .try_into()
            .map_err(|_| PckError::InvalidValue("data_format".into()))?;
        let width_2 = u16::read_ne(r)?;
        let height_2 = u16::read_ne(r)?;
        let mipmap_count = u32::read_ne(r)?;
        let image_format: ImageFormat = u32::read_ne(r)?
            .try_into()
            .map_err(|_| PckError::InvalidValue("image_format".into()))?;

        let header = CompressedTexture2dHeader {
            version,
            width,
            height,
            format_flags,
            mipmap_limit,
            data_format,
            width_2,
            height_2,
            mipmaps: mipmap_count,
            image_format,
        };

        let mut images = vec![];
        match data_format {
            CompressedTexture2dDataFormat::Image => match image_format {
                ImageFormat::FORMAT_DXT1
                | ImageFormat::FORMAT_DXT3
                | ImageFormat::FORMAT_DXT5
                | ImageFormat::FORMAT_RGTC_R
                | ImageFormat::FORMAT_RGTC_RG
                | ImageFormat::FORMAT_BPTC_RGBA
                | ImageFormat::FORMAT_BPTC_RGBF
                | ImageFormat::FORMAT_BPTC_RGBFU => {
                    // calculate remaining bytes in r so we can use "take" to generate a Seek impl
                    // that starts at position 0, which is required for our "SeekingChain" impl
                    let remaining = {
                        let pos = r.stream_position()?;
                        let len = r.seek(SeekFrom::End(0))?;
                        r.seek(SeekFrom::Start(pos))?;
                        len - pos
                    };

                    let size = 124u32.to_le_bytes(); // must be 124
                    let flags = (0x1u32 | 0x2 | 0x4 | 0x1000 | 0x20000).to_le_bytes();
                    let height = height.to_le_bytes();
                    let width = width.to_le_bytes();
                    let pols = (remaining as u32).to_le_bytes();
                    let depth = 0u32.to_le_bytes();
                    let mipmaps = (mipmap_count + 1).to_le_bytes();
                    let pf_size = 32u32.to_le_bytes(); // must be 32
                    let pf_flags = 0x4u32.to_le_bytes(); // enable fourcc
                    let rgb_bits = 0u32.to_le_bytes();
                    let r_mask = 0u32.to_le_bytes();
                    let g_mask = 0u32.to_le_bytes();
                    let b_mask = 0u32.to_le_bytes();
                    let a_mask = 0u32.to_le_bytes();
                    let caps = (if mipmap_count > 0 {
                        0x8 | 0x400000
                    } else {
                        0u32
                    })
                    .to_le_bytes();
                    let caps2 = 0u32.to_le_bytes();
                    let dxgi = (match image_format {
                        ImageFormat::FORMAT_DXT1 => 71u32,    // BC1
                        ImageFormat::FORMAT_DXT3 => 74,       // BC2
                        ImageFormat::FORMAT_DXT5 => 77,       // BC3
                        ImageFormat::FORMAT_RGTC_R => 80,     // BC4
                        ImageFormat::FORMAT_RGTC_RG => 83,    // BC5
                        ImageFormat::FORMAT_BPTC_RGBA => 98,  // BC7
                        ImageFormat::FORMAT_BPTC_RGBF => 96,  // BC6 Signed
                        ImageFormat::FORMAT_BPTC_RGBFU => 95, // BC6 Unsigned
                        _ => unreachable!(),
                    })
                    .to_le_bytes();
                    let res_dim = 3u32.to_le_bytes(); // 2d texture
                    let misc_flags = 0u32.to_le_bytes();
                    let array_size = 1u32.to_le_bytes(); // must be 1
                    let misc_flags2 = 0u32.to_le_bytes();
                    let dds_header = [
                        b'D',
                        b'D',
                        b'S',
                        b' ', // magic
                        size[0],
                        size[1],
                        size[2],
                        size[3], // size
                        flags[0],
                        flags[1],
                        flags[2],
                        flags[3], // flags
                        height[0],
                        height[1],
                        height[2],
                        height[3], // height
                        width[0],
                        width[1],
                        width[2],
                        width[3], // width
                        pols[0],
                        pols[1],
                        pols[2],
                        pols[3], // pitchOrLinearSize
                        depth[0],
                        depth[1],
                        depth[2],
                        depth[3], // depth
                        mipmaps[0],
                        mipmaps[1],
                        mipmaps[2],
                        mipmaps[3], // mipmap count
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0, // 11 * 4 reserved [reserved1]
                        pf_size[0],
                        pf_size[1],
                        pf_size[2],
                        pf_size[3], // pixelformat size
                        pf_flags[0],
                        pf_flags[1],
                        pf_flags[2],
                        pf_flags[3], // pixelformat flags
                        b'D',
                        b'X',
                        b'1',
                        b'0', // fourcc
                        rgb_bits[0],
                        rgb_bits[1],
                        rgb_bits[2],
                        rgb_bits[3], // rgb bit count
                        r_mask[0],
                        r_mask[1],
                        r_mask[2],
                        r_mask[3], // r bit mask
                        g_mask[0],
                        g_mask[1],
                        g_mask[2],
                        g_mask[3], // g bit mask
                        b_mask[0],
                        b_mask[1],
                        b_mask[2],
                        b_mask[3], // b bit mask
                        a_mask[0],
                        a_mask[1],
                        a_mask[2],
                        a_mask[3], // a bit mask
                        caps[0],
                        caps[1],
                        caps[2],
                        caps[3], // caps
                        caps2[0],
                        caps2[1],
                        caps2[2],
                        caps2[3], // caps2
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0, // 3 * 4 reserved [caps3, cap4, reserved2]
                        dxgi[0],
                        dxgi[1],
                        dxgi[2],
                        dxgi[3], // dxgi format
                        res_dim[0],
                        res_dim[1],
                        res_dim[2],
                        res_dim[3], // resource dimension
                        misc_flags[0],
                        misc_flags[1],
                        misc_flags[2],
                        misc_flags[3], // misc flags
                        array_size[0],
                        array_size[1],
                        array_size[2],
                        array_size[3], // array size
                        misc_flags2[0],
                        misc_flags2[1],
                        misc_flags2[2],
                        misc_flags2[3], // misc flags 2
                    ];

                    let data_reader = SeekingChain {
                        first: Cursor::new(dds_header),
                        second: r.take(remaining),
                        done_first: false,
                    };
                    // TODO: use image when it supports mipmaps
                    /*let image = image::ImageReader::new(data_reader)
                    .with_guessed_format()?
                    .decode()?;*/
                    let mut decoder = dds::Decoder::new(data_reader)?;
                    while !decoder.is_done() {
                        let Some(info) = decoder.surface_info() else {
                            unreachable!();
                        };

                        let size = info.size();
                        let mut buf = vec![
                            0;
                            size.pixels() as usize
                                * ColorFormat::RGBA_U8.bytes_per_pixel() as usize
                        ];
                        let image_view =
                            ImageViewMut::new(&mut buf, size, ColorFormat::RGBA_U8).unwrap();
                        decoder.read_surface(image_view)?;

                        images.push(
                            image::RgbaImage::from_vec(size.width, size.height, buf)
                                .unwrap()
                                .into(),
                        );
                    }

                    if images.len() != (mipmap_count + 1) as usize {
                        return Err(PckError::InvalidValue(format!(
                            "surface count mismatch, expected {} but expected {}",
                            mipmap_count + 1,
                            images.len()
                        )));
                    }
                }
                _ => {
                    return Err(PckError::NotImplemented(format!(
                        "CompressedTexture2d: image_format ({header:?})"
                    )));
                }
            },
            CompressedTexture2dDataFormat::Png | CompressedTexture2dDataFormat::WebP => {
                for _ in 0..=mipmap_count {
                    let size = u32::read_ne(r)?;
                    let mut data_reader = r.take(size as _);
                    let image = image::ImageReader::new(&mut data_reader)
                        .with_guessed_format()?
                        .decode()?;
                    data_reader.seek(SeekFrom::End(0))?;
                    images.push(image);
                }
            }
            _ => {
                return Err(PckError::NotImplemented(format!(
                    "CompressedTexture2d: data_format ({header:?})"
                )));
            }
        }

        Ok(Self { header, images })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CompressedTexture2dHeader {
    pub version: CompressedTexture2dVersion,
    pub width: u32,
    pub height: u32,
    pub format_flags: CompressedTexture2dDataFormatFlags,
    pub mipmap_limit: u32,
    pub data_format: CompressedTexture2dDataFormat,
    pub width_2: u16,
    pub height_2: u16,
    pub mipmaps: u32,
    pub image_format: ImageFormat,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum CompressedTexture2dVersion {
    V1 = 1,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum CompressedTexture2dDataFormat {
    Image,
    Png,
    WebP,
    BasisUniversal,
}

bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct CompressedTexture2dDataFormatFlags : u32 {
        const STREAM = 1 << 22;
        const HAS_MIPMAPS = 1 << 23;
        const DETECT_3D = 1 << 24;
        const DETECT_SRGB = 1 << 25;
        const DETECT_NORMAL = 1 << 26;
        const DETECT_ROUGNESS = 1 << 27;
    }
}

/// From Godot source
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum ImageFormat {
    FORMAT_L8,  // Luminance
    FORMAT_LA8, // Luminance-Alpha
    FORMAT_R8,
    FORMAT_RG8,
    FORMAT_RGB8,
    FORMAT_RGBA8,
    FORMAT_RGBA4444,
    FORMAT_RGB565,
    FORMAT_RF, // Float
    FORMAT_RGF,
    FORMAT_RGBF,
    FORMAT_RGBAF,
    FORMAT_RH, // Half
    FORMAT_RGH,
    FORMAT_RGBH,
    FORMAT_RGBAH,
    FORMAT_RGBE9995,
    FORMAT_DXT1,       // BC1
    FORMAT_DXT3,       // BC2
    FORMAT_DXT5,       // BC3
    FORMAT_RGTC_R,     // BC4
    FORMAT_RGTC_RG,    // BC5
    FORMAT_BPTC_RGBA,  // BC7
    FORMAT_BPTC_RGBF,  // BC6 Signed
    FORMAT_BPTC_RGBFU, // BC6 Unsigned
    FORMAT_ETC,        // ETC1
    FORMAT_ETC2_R11,
    FORMAT_ETC2_R11S, // Signed, NOT srgb.
    FORMAT_ETC2_RG11,
    FORMAT_ETC2_RG11S, // Signed, NOT srgb.
    FORMAT_ETC2_RGB8,
    FORMAT_ETC2_RGBA8,
    FORMAT_ETC2_RGB8A1,
    FORMAT_ETC2_RA_AS_RG, // ETC2 RGBA with a RA-RG swizzle for normal maps.
    FORMAT_DXT5_RA_AS_RG, // BC3 with a RA-RG swizzle for normal maps.
    FORMAT_ASTC_4x4,
    FORMAT_ASTC_4x4_HDR,
    FORMAT_ASTC_8x8,
    FORMAT_ASTC_8x8_HDR,
    FORMAT_R16,
    FORMAT_RG16,
    FORMAT_RGB16,
    FORMAT_RGBA16,
    FORMAT_R16I,
    FORMAT_RG16I,
    FORMAT_RGB16I,
    FORMAT_RGBA16I,
}

struct SeekingChain<T, U> {
    first: T,
    second: U,
    done_first: bool,
}

impl<T: Read, U: Read> Read for SeekingChain<T, U> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if !self.done_first {
            match self.first.read(buf)? {
                0 if !buf.is_empty() => self.done_first = true,
                n => return Ok(n),
            }
        }
        self.second.read(buf)
    }
}

impl<T: BufRead, U: BufRead> BufRead for SeekingChain<T, U> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        if !self.done_first {
            match self.first.fill_buf()? {
                [] => self.done_first = true,
                buf => return Ok(buf),
            }
        }
        self.second.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        if !self.done_first {
            self.first.consume(amt)
        } else {
            self.second.consume(amt)
        }
    }
}

impl<T: Seek, U: Seek> Seek for SeekingChain<T, U> {
    fn seek(&mut self, target_pos: SeekFrom) -> std::io::Result<u64> {
        let first_pos = self.first.stream_position()?;
        let first_len = {
            let len = self.first.seek(SeekFrom::End(0))?;
            self.first.seek(SeekFrom::Start(first_pos))?;
            len
        };
        let second_pos = self.second.stream_position()?;
        let second_len = {
            let len = self.second.seek(SeekFrom::End(0))?;
            self.second.seek(SeekFrom::Start(second_pos))?;
            len
        };
        let total_len = first_len
            .checked_add(second_len)
            .ok_or(std::io::ErrorKind::FileTooLarge)?;

        let target_pos = match target_pos {
            SeekFrom::Start(n) => Some(n),
            SeekFrom::End(n) => total_len.checked_add_signed(n),
            SeekFrom::Current(n) => {
                let current_pos = if self.done_first {
                    first_len + self.second.stream_position()?
                } else {
                    self.first.stream_position()?
                };
                current_pos.checked_add_signed(n)
            }
        };
        let target_pos = match target_pos {
            Some(v) if v <= total_len => v,
            _ => return Err(std::io::ErrorKind::InvalidInput.into()),
        };

        if target_pos <= first_len {
            self.done_first = false;
            self.second.seek(SeekFrom::Start(0))?;
            self.first.seek(SeekFrom::Start(target_pos))
        } else {
            self.done_first = true;
            self.first.seek(SeekFrom::End(0))?;
            Ok(first_len + self.second.seek(SeekFrom::Start(target_pos - first_len))?)
        }
    }
}
