pub mod error;
pub mod format;

mod gdf;
mod macros;

use std::io;

use better_default::Default;
use byteorder::{LE, ReadBytesExt};

use crate::error::{Error, Result};
use crate::format::TexFormat;

#[derive(Debug, Clone, Default)]
pub struct TexHeader {
    magic: [u8; 4],
    version: u32,
    width: u16,
    height: u16,
    depth: u16,
    mipmap_count: u8,
    tex_count: u8,
    mipmap_header_size: u8,
    #[default(TexFormat::Bc7Unorm)]
    format: TexFormat,
    swizzle_control: i32,
    cubemap_marker: u32,
    unkn04: u8,
    unkn05: u8,
    null0: u16,
    // swizzle data
    swizzle_height_depth: u8,
    swizzle_width: u8,
    null1: u16,
    seven: u16,
    one: u16,
}

impl TexHeader {
    const MAGIC: [u8; 4] = [0x54, 0x45, 0x58, 0x00];

    pub fn from_reader<R>(reader: &mut R) -> Result<Self>
    where
        R: io::Read,
    {
        let mut this = TexHeader::default();
        reader.read_exact(&mut this.magic)?;
        if this.magic != Self::MAGIC {
            return Err(Error::NotTexFile);
        }

        this.version = reader.read_u32::<LE>()?;
        this.width = reader.read_u16::<LE>()?;
        this.height = reader.read_u16::<LE>()?;
        this.depth = reader.read_u16::<LE>()?;

        if this.version > 11 && this.version != 190820018 {
            this.tex_count = reader.read_u8()?;
            this.mipmap_header_size = reader.read_u8()?;
            this.mipmap_count = this.mipmap_header_size / MipEntry::SIZE as u8;
        } else {
            this.mipmap_count = reader.read_u8()?;
            this.tex_count = reader.read_u8()?;
        }

        let format_var = reader.read_u32::<LE>()?;
        let format =
            TexFormat::from_repr(format_var).ok_or(Error::UnsupportedTexFormat(format_var))?;
        this.format = format;
        this.swizzle_control = reader.read_i32::<LE>()?;
        this.cubemap_marker = reader.read_u32::<LE>()?;
        this.unkn04 = reader.read_u8()?;
        this.unkn05 = reader.read_u8()?;
        this.null0 = reader.read_u16::<LE>()?;

        if this.version > 27 && this.version != 190820018 {
            // swizzle data
            this.swizzle_height_depth = reader.read_u8()?;
            this.swizzle_width = reader.read_u8()?;
            this.null1 = reader.read_u16::<LE>()?;
            this.seven = reader.read_u16::<LE>()?;
            this.one = reader.read_u16::<LE>()?;
        }

        if this.swizzle_control == 1 {
            return Err(Error::Unimplemented(
                "Swizzle not implemented yet.".to_string(),
            ));
        }

        Ok(this)
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MipEntry {
    offset: u64,
    scanline_length: u32,
    uncompressed_size: u32,
}

impl MipEntry {
    const SIZE: usize = size_of::<MipEntry>();

    pub fn from_reader<R>(reader: &mut R) -> Result<Self>
    where
        R: io::Read,
    {
        let mut buf = [0; Self::SIZE];
        reader.read_exact(&mut buf)?;
        unsafe { Ok(std::mem::transmute::<[u8; 16], MipEntry>(buf)) }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CompressionInfo {
    compressed_size: u32,
    compressed_offset: u32,
}

impl CompressionInfo {
    const SIZE: usize = size_of::<CompressionInfo>();

    pub fn from_reader<R>(reader: &mut R) -> Result<Self>
    where
        R: io::Read,
    {
        let mut buf = [0; Self::SIZE];
        reader.read_exact(&mut buf)?;
        unsafe { Ok(std::mem::transmute::<[u8; 8], CompressionInfo>(buf)) }
    }
}

#[derive(Clone)]
pub struct MipData {
    entry: MipEntry,
    texture_data: Vec<u8>,
}

impl std::fmt::Debug for MipData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MipData")
            .field("entry", &self.entry)
            .field("texture_data.len()", &self.texture_data.len())
            .finish()
    }
}

impl MipData {
    pub fn new(entry: MipEntry, texture_data: Vec<u8>) -> Self {
        Self {
            entry,
            texture_data,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tex {
    pub(crate) header: TexHeader,
    pub(crate) mip_datas: Vec<MipData>,
    // // wilds
    // image_header_list: Vec<u8>,
}

impl Tex {
    pub fn from_reader<R>(reader: &mut R) -> Result<Self>
    where
        R: io::Read + io::Seek,
    {
        let header = TexHeader::from_reader(reader)?;

        let num_mipmaps = header.mipmap_count as usize * header.tex_count as usize;
        let mut mip_entries = Vec::with_capacity(num_mipmaps);
        for _ in 0..num_mipmaps {
            mip_entries.push(MipEntry::from_reader(reader)?);
        }
        let mut compression_infos = Vec::with_capacity(num_mipmaps);
        for _ in 0..num_mipmaps {
            compression_infos.push(CompressionInfo::from_reader(reader)?);
        }

        if mip_entries.len() != compression_infos.len() {
            return Err(Error::Internal(
                "mip_entries and compression_infos have different lengths".to_string(),
            ));
        }

        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        // read mipmap data
        let mut mipmaps = Vec::with_capacity((header.tex_count * header.mipmap_count) as usize);
        for (mip_entry, compression_info) in mip_entries.iter().zip(compression_infos.iter()) {
            let start = compression_info.compressed_offset as usize;
            let end = start + compression_info.compressed_size as usize;
            let padding = if header.swizzle_control != 1 {
                0
            } else {
                mip_entry.uncompressed_size - mip_entry.scanline_length
            };
            let mut mip_data = data[start..end].to_vec();
            if padding > 0 {
                mip_data.extend(vec![0u8; padding as usize]);
            }
            mipmaps.push(mip_data);
        }

        for mip_data in &mut mipmaps {
            if mip_data.len() >= 2 && mip_data[0..2] == [0x04, 0xFB] {
                // decompress mipmap data
                *mip_data = Self::decompress_mipmap(mip_data)?;
            }
        }

        // validate
        let mut _idx = 0;
        for _ in 0..header.tex_count {
            for img_mip_idx in 0..header.mipmap_count {
                let mip_data = &mipmaps[_idx];
                let mip_entry = &mip_entries[_idx];
                if mip_data.len()
                    != mip_entry.uncompressed_size as usize
                        * u32::max(1, header.depth as u32 >> img_mip_idx) as usize
                {
                    return Err(Error::Internal(
                        "Mipmap data size does not match mipmap entry size".to_string(),
                    ));
                }
                _idx += 1;
            }
        }

        Ok(Tex {
            header,
            mip_datas: mipmaps
                .into_iter()
                .zip(mip_entries)
                .map(|(data, entry)| MipData::new(entry, data))
                .collect(),
        })
    }

    pub fn header(&self) -> &TexHeader {
        &self.header
    }

    pub fn mip_datas(&self) -> &[MipData] {
        self.mip_datas.as_slice()
    }

    fn decompress_mipmap(mip_data: &[u8]) -> Result<Vec<u8>> {
        let mut decompressor = gdf::GDfDecompressor::new()?;
        let out_data = decompressor.decompress(mip_data)?;

        Ok(out_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FILE_1: &str = "test_files/ch04_000_0000_1002_MB.tex.241106027";

    #[test]
    fn test1() {
        let data = std::fs::read(TEST_FILE_1).unwrap();
        let mut reader = std::io::Cursor::new(data);
        let tex = Tex::from_reader(&mut reader).unwrap();
        eprintln!("{:#?}", tex);
    }
}
