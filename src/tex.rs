use std::borrow::Cow;
use std::io::{self, Write as _};

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use ddsfile::{AlphaMode, D3D10ResourceDimension, Dds, DxgiFormat, NewDxgiParams};
use num_traits::FromPrimitive;

use crate::error::{Error, Result};
use crate::format::TexFormat;
use crate::gdf;

#[derive(Debug, Clone, better_default::Default)]
pub struct TexHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub width: u16,
    pub height: u16,
    pub depth: u16,
    pub mipmap_count: u8,
    pub tex_count: u8,
    pub mipmap_header_size: u8,
    #[default(TexFormat::Bc7Unorm)]
    pub format: TexFormat,
    pub swizzle_control: i32,
    pub cubemap_marker: u32,
    unkn04: u8,
    unkn05: u8,
    null0: u16,
    // swizzle data
    pub swizzle_height_depth: u8,
    pub swizzle_width: u8,
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

    pub fn as_bytes(&self) -> Result<Vec<u8>> {
        let buf: Vec<u8> = Vec::new();
        let mut writer = io::Cursor::new(buf);

        writer.write_all(&Self::MAGIC)?;
        writer.write_u32::<LE>(self.version)?;
        writer.write_u16::<LE>(self.width)?;
        writer.write_u16::<LE>(self.height)?;
        writer.write_u16::<LE>(self.depth)?;

        if self.version > 11 && self.version != 190820018 {
            writer.write_u8(self.tex_count)?;
            writer.write_u8(self.mipmap_header_size)?;
        } else {
            writer.write_u8(self.mipmap_count)?;
            writer.write_u8(self.tex_count)?;
        }

        writer.write_u32::<LE>(self.format as u32)?;
        writer.write_i32::<LE>(self.swizzle_control)?;
        writer.write_u32::<LE>(self.cubemap_marker)?;
        writer.write_u8(self.unkn04)?;
        writer.write_u8(self.unkn05)?;
        writer.write_u16::<LE>(self.null0)?;

        if self.version > 27 && self.version != 190820018 {
            writer.write_u8(self.swizzle_height_depth)?;
            writer.write_u8(self.swizzle_width)?;
            writer.write_u16::<LE>(self.null1)?;
            writer.write_u16::<LE>(self.seven)?;
            writer.write_u16::<LE>(self.one)?;
        }

        Ok(writer.into_inner())
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

    pub fn as_bytes(&self) -> [u8; Self::SIZE] {
        unsafe { std::mem::transmute::<MipEntry, [u8; 16]>(self.clone()) }
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

    pub fn as_bytes(&self) -> [u8; Self::SIZE] {
        unsafe { std::mem::transmute::<CompressionInfo, [u8; 8]>(self.clone()) }
    }
}

#[derive(Clone)]
pub struct MipData {
    pub entry: MipEntry,
    pub compression_info: CompressionInfo,
    pub texture_data: Vec<u8>,

    pub is_gdeflate: bool,
}

impl std::fmt::Debug for MipData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MipData")
            .field("entry", &self.entry)
            .field("compression_info", &self.compression_info)
            .field("texture_data.len()", &self.texture_data.len())
            .field("is_gdeflate", &self.is_gdeflate)
            .finish()
    }
}

impl MipData {
    pub fn new(entry: MipEntry, compression_info: CompressionInfo, texture_data: Vec<u8>) -> Self {
        let is_gdeflate = texture_data.len() >= 2 && texture_data[0..2] == [0x04, 0xFB];
        Self {
            entry,
            compression_info,
            texture_data,
            is_gdeflate,
        }
    }

    pub fn uncompressed_data(
        &self,
        decompressor: Option<&mut gdf::GDfDecompressor>,
    ) -> Result<Cow<Vec<u8>>> {
        if self.is_gdeflate {
            // let mut decompressor = gdf::GDfDecompressor::new()?;
            let decompressor = if let Some(decompressor) = decompressor {
                decompressor
            } else {
                &mut gdf::GDfDecompressor::new()?
            };
            let out_data = decompressor.decompress(&self.texture_data)?;
            Ok(Cow::Owned(out_data))
        } else {
            Ok(Cow::Borrowed(&self.texture_data))
        }
    }

    pub fn is_compressed(&self) -> bool {
        self.is_gdeflate
    }
}

#[derive(Debug, Clone)]
pub struct Tex {
    pub header: TexHeader,
    pub mip_datas: Vec<MipData>,
    // // wilds
    // image_header_list: Vec<u8>,
}

impl Tex {
    pub fn from_reader<R>(reader: &mut R) -> Result<Self>
    where
        R: io::Read,
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

        // collect MipData
        let mip_datas: Vec<MipData> = mipmaps
            .into_iter()
            .zip(mip_entries)
            .zip(compression_infos)
            .map(|((data, entry), compression_info)| MipData::new(entry, compression_info, data))
            .collect();

        // validate
        let mut _idx = 0;
        for _ in 0..header.tex_count {
            for img_mip_idx in 0..header.mipmap_count {
                let mip_data = &mip_datas[_idx];
                if mip_data.uncompressed_data(None)?.len()
                    != mip_data.entry.uncompressed_size as usize
                        * u32::max(1, header.depth as u32 >> img_mip_idx) as usize
                {
                    return Err(Error::Internal(
                        "Mipmap data size does not match mipmap entry size".to_string(),
                    ));
                }
                _idx += 1;
            }
        }

        Ok(Tex { header, mip_datas })
    }

    /// Create a new Tex file data.
    pub fn as_bytes(self) -> Result<Vec<u8>> {
        let buf: Vec<u8> = Vec::new();
        let mut writer = io::Cursor::new(buf);

        writer.write_all(&self.header.as_bytes()?)?;
        // write mipmap entries and compression infos
        for mip_data in &self.mip_datas {
            writer.write_all(&mip_data.entry.as_bytes())?;
        }
        for mip_data in &self.mip_datas {
            writer.write_all(&mip_data.compression_info.as_bytes())?;
        }
        // write mipmap data
        for mip_data in &self.mip_datas {
            writer.write_all(&mip_data.texture_data)?;
        }

        Ok(writer.into_inner())
    }

    /// Decompress all mipmaps.
    pub fn batch_decompress(&mut self) -> Result<()> {
        let mut decompressor = gdf::GDfDecompressor::new()?;

        let mut curr_comp_offset = 0;
        for mip_data in &mut self.mip_datas {
            if !mip_data.is_compressed() {
                // fix header
                mip_data.compression_info.compressed_offset = curr_comp_offset;
                curr_comp_offset += mip_data.compression_info.compressed_size;
                continue;
            }
            let out_data = decompressor.decompress(&mip_data.texture_data)?;
            // fix header
            mip_data.compression_info.compressed_offset = curr_comp_offset;
            mip_data.compression_info.compressed_size = out_data.len() as u32;
            mip_data.is_gdeflate = false;
            curr_comp_offset += out_data.len() as u32;

            mip_data.texture_data = out_data;
        }
        Ok(())
    }

    pub fn to_dds(&self, mipmap_count: usize) -> Result<Dds> {
        if mipmap_count > self.mip_datas.len() {
            return Err(Error::Internal("mipmap_count is out of range".to_string()));
        }

        // TODO: swizzle
        let mipmaps = &self.mip_datas[0..mipmap_count];

        let mut dds = Dds::new_dxgi(NewDxgiParams {
            height: self.header.height as u32,
            width: self.header.width as u32,
            depth: Some(self.header.depth as u32),
            format: DxgiFormat::from_u32(self.header.format as u32).unwrap(),
            mipmap_levels: Some(mipmap_count as u32),
            array_layers: None,
            caps2: None,
            is_cubemap: self.header.cubemap_marker != 0,
            resource_dimension: D3D10ResourceDimension::Texture2D,
            alpha_mode: AlphaMode::Unknown,
        })?;

        // decompress if needed
        let has_compressed = mipmaps.iter().any(|mip_data| mip_data.is_compressed());
        let mut decompressor = if has_compressed {
            Some(gdf::GDfDecompressor::new()?)
        } else {
            None
        };

        let mut data: Vec<u8> = Vec::new();
        for mip_data in mipmaps {
            if mip_data.is_compressed() {
                data.extend(mip_data.uncompressed_data(decompressor.as_mut())?.as_ref());
            } else {
                data.extend(mip_data.texture_data.as_slice());
            }
        }
        dds.data = data;

        Ok(dds)
    }

    /// Convert to Image struct.
    #[cfg(feature = "image")]
    pub fn to_rgba_image(&self, mipmap_idx: usize) -> Result<image::RgbaImage> {
        if mipmap_idx >= self.mip_datas.len() {
            return Err(Error::Internal("mipmap_idx is out of range".to_string()));
        }

        let dds = self.to_dds(mipmap_idx + 1)?;
        let rgba_image = image_dds::image_from_dds(&dds, mipmap_idx as u32)?;

        Ok(rgba_image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FILE_GDF: &str = "test_files/ch04_000_0000_1002_MB.tex.241106027";
    const TEST_FILE_NO_GDF: &str = "test_files/uncompress_ch04_000_0000_1001_ALBD.tex.241106027";

    #[test]
    fn test_parse_tex() {
        let data = std::fs::read(TEST_FILE_NO_GDF).unwrap();
        let mut reader = std::io::Cursor::new(data);
        let tex = Tex::from_reader(&mut reader).unwrap();
        eprintln!("{:#?}", tex);
    }

    #[test]
    fn test_parse_tex_gdf() {
        let data = std::fs::read(TEST_FILE_GDF).unwrap();
        let mut reader = std::io::Cursor::new(data);
        let tex = Tex::from_reader(&mut reader).unwrap();
        eprintln!("{:#?}", tex);
    }

    #[test]
    fn test_tex_header_rw() {
        let mut data = std::fs::read(TEST_FILE_GDF).unwrap();
        let mut reader = std::io::Cursor::new(&mut data);
        let header = TexHeader::from_reader(&mut reader).unwrap();
        let bytes = header.as_bytes().unwrap();
        assert_eq!(bytes, &data[0..bytes.len()]);
    }

    #[test]
    fn test_tex_rw() {
        let mut data = std::fs::read(TEST_FILE_GDF).unwrap();
        let mut reader = std::io::Cursor::new(&mut data);
        let tex = Tex::from_reader(&mut reader).unwrap();
        let bytes = tex.as_bytes().unwrap();
        assert_eq!(data, bytes);
    }

    #[test]
    fn test_1() {
        let mut data = std::fs::read("test_files/ch04_000_0000_1001_ALBD.tex.241106027").unwrap();
        let mut reader = std::io::Cursor::new(&mut data);
        let mut tex = Tex::from_reader(&mut reader).unwrap();
        tex.batch_decompress().unwrap();
        let bytes = tex.as_bytes().unwrap();
        std::fs::write("test_files/uncompress_.tex.241106027", &bytes).unwrap();

        // read again
        let mut reader = std::io::Cursor::new(bytes);
        let tex = Tex::from_reader(&mut reader).unwrap();
        eprintln!("{:#?}", tex);
    }

    #[test]
    fn test_tex_to_dds() {
        let mut data = std::fs::read("test_files/ch04_000_0000_1001_ALBD.tex.241106027").unwrap();
        let mut reader = std::io::Cursor::new(&mut data);
        let tex = Tex::from_reader(&mut reader).unwrap();
        tex.to_dds(tex.header.mipmap_count as usize).unwrap();
    }

    #[cfg(feature = "image")]
    #[test]
    fn test_tex_to_rgba_image() {
        let mut data = std::fs::read("test_files/ch04_000_0000_1001_ALBD.tex.241106027").unwrap();
        let mut reader = std::io::Cursor::new(&mut data);
        let tex = Tex::from_reader(&mut reader).unwrap();
        let image = tex.to_rgba_image(0).unwrap();
        image
            .save("test_files/ch04_000_0000_1001_ALBD.png")
            .unwrap();
    }
}
