//! GDeflate compression and decompression.
//!
//! Reference: https://github.com/microsoft/DirectStorage/blob/main/GDeflate/GDeflate/GDeflateCompress.cpp

use std::io;

use byteorder::{LE, ReadBytesExt};
use gdeflate::sys;

use crate::macros::BitField as _;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Decompression error: {0}")]
    Decompression(#[from] DecompressionError),
    #[error("Compression error: {0}")]
    Compression(#[from] CompressionError),
}

#[derive(Debug, thiserror::Error)]
pub enum DecompressionError {
    #[error("Bad data.")]
    BadData,
    #[error("Decompressor creation failed.")]
    DecompressorCreationFailed,
    #[error("Decompression failed.")]
    DecompressionFailed,
    #[error("Insufficient space.")]
    InsufficientSpace,
}

#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
    #[error("Compressor creation failed.")]
    CompressorCreationFailed,
    #[error("Compression failed.")]
    CompressionFailed,
}

// partial from https://github.com/c-ola/libdeflater
// modified

const K_GDEFLATE_ID: u8 = 4;
const KDEFAULT_TILE_SIZE: usize = 64 * 1024;

#[derive(Debug)]
struct TileStream {
    id: u8,
    magic: u8,
    num_tiles: u16,      //u16,
    tile_size_idx: u32,  //u8,
    last_tile_size: u32, //u32,
    reserv1: u32,
    //bitfield: u32, //tileSizeIdx: 2, lastTileSize: 18, reserv1: 12
}

impl TileStream {
    pub fn new(uncompressed_size: usize) -> TileStream {
        let mut num_tiles = (uncompressed_size / KDEFAULT_TILE_SIZE).try_into().unwrap();
        let last_tile_size = (uncompressed_size - num_tiles as usize * KDEFAULT_TILE_SIZE)
            .try_into()
            .unwrap();
        num_tiles += if last_tile_size != 0 { 1 } else { 0 };
        TileStream {
            id: K_GDEFLATE_ID,
            magic: K_GDEFLATE_ID ^ 0xFF,
            tile_size_idx: 1,
            num_tiles,
            last_tile_size,
            reserv1: 0,
        }
    }

    pub fn from<W: io::Read + io::Seek>(data: &mut W) -> Result<TileStream> {
        let id = data.read_u8()?;
        let magic = data.read_u8()?;
        let num_tiles = data.read_u16::<LE>()?;
        let flags = data.read_u32::<LE>()?;
        let (tile_size_idx, last_tile_size, reserv1) = flags.bit_split((2, 18, 12));
        Ok(TileStream {
            id,
            magic,
            num_tiles,
            tile_size_idx,
            last_tile_size,
            reserv1,
        })
    }

    pub fn get_uncompressed_size(&self) -> usize {
        self.num_tiles as usize * KDEFAULT_TILE_SIZE
            - if self.last_tile_size == 0 {
                0
            } else {
                KDEFAULT_TILE_SIZE - self.last_tile_size as usize
            }
    }

    pub fn is_valid(&self) -> bool {
        self.id == self.magic ^ 0xFF && self.id == K_GDEFLATE_ID
    }
}

pub struct GDfDecompressor(*mut sys::libdeflate_gdeflate_decompressor);

impl GDfDecompressor {
    pub fn new() -> Result<GDfDecompressor> {
        let decompressor = unsafe { sys::libdeflate_alloc_gdeflate_decompressor() };
        if decompressor.is_null() {
            Err(DecompressionError::DecompressorCreationFailed)?
        } else {
            Ok(Self(decompressor))
        }
    }

    pub fn decompress(&mut self, in_data: &[u8]) -> Result<Vec<u8>> {
        let tile_stream = TileStream::from(&mut io::Cursor::new(in_data))?;
        if !tile_stream.is_valid() {
            return Err(DecompressionError::BadData)?;
        }
        let uncompressed_size = tile_stream.get_uncompressed_size();

        let mut out_data = vec![0u8; uncompressed_size];
        let mut _bytes_read = 0;

        unsafe {
            let tile_offsets = in_data.as_ptr().add(size_of::<u64>()) as *const u32;

            let base_in_ptr = tile_offsets.add(tile_stream.num_tiles as usize) as *const u8;

            for tile_index in 0..tile_stream.num_tiles {
                let tile_offset = if tile_index > 0 {
                    *tile_offsets.add(tile_index as usize)
                } else {
                    0
                } as usize;

                let data_size = if tile_index < tile_stream.num_tiles - 1 {
                    *tile_offsets.add(tile_index as usize + 1) - tile_offset as u32
                } else {
                    *tile_offsets
                };
                let data_ptr = base_in_ptr.add(tile_offset) as *const std::ffi::c_void;

                let mut compressed_page = sys::libdeflate_gdeflate_in_page {
                    data: data_ptr,
                    nbytes: data_size as usize,
                };

                let output_offset = tile_index as usize * KDEFAULT_TILE_SIZE;
                let out_ptr = out_data.as_mut_ptr().add(output_offset) as *mut std::ffi::c_void;
                let mut out_nbytes = 0;
                let decomp_result: sys::libdeflate_result = sys::libdeflate_gdeflate_decompress(
                    self.0,
                    &mut compressed_page,
                    1,
                    out_ptr,
                    KDEFAULT_TILE_SIZE,
                    &mut out_nbytes,
                )
                    as sys::libdeflate_result;
                match decomp_result {
                    sys::libdeflate_result_LIBDEFLATE_SUCCESS => _bytes_read += out_nbytes,
                    sys::libdeflate_result_LIBDEFLATE_BAD_DATA => {
                        return Err(DecompressionError::BadData)?;
                    }
                    sys::libdeflate_result_LIBDEFLATE_INSUFFICIENT_SPACE => {
                        return Err(DecompressionError::InsufficientSpace)?;
                    }
                    _ => {
                        panic!(
                            "libdeflate_gdeflate_decompress returned an unknown error type: this is an internal bug that **must** be fixed"
                        );
                    }
                }
            }
        }

        Ok(out_data)
    }
}

impl Drop for GDfDecompressor {
    fn drop(&mut self) {
        unsafe {
            sys::libdeflate_free_gdeflate_decompressor(self.0);
        }
    }
}

struct CompressionContext {
    input_ptr: &'static [u8],
    input_size: usize,
    tiles: Vec<Vec<u8>>,
    num_items: u32,
}

pub struct GDfCompressor(*mut sys::libdeflate_gdeflate_compressor);

impl GDfCompressor {
    pub fn new(level: gdeflate::CompressionLevel) -> Result<GDfCompressor> {
        let compressor = unsafe { sys::libdeflate_alloc_gdeflate_compressor(level as i32) };
        if compressor.is_null() {
            Err(CompressionError::CompressorCreationFailed)?
        } else {
            Ok(Self(compressor))
        }
    }

    fn compress_tile(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        let mut page_count = 0;
        let scratch_size = unsafe {
            sys::libdeflate_gdeflate_compress_bound(self.0, input.len(), &mut page_count)
        };
        assert_eq!(page_count, 1);

        let mut scratch_buffer = vec![0u8; scratch_size];
        let mut compressed_page = sys::libdeflate_gdeflate_out_page {
            data: scratch_buffer.as_mut_ptr() as *mut std::ffi::c_void,
            nbytes: scratch_size,
        };

        let result = unsafe {
            sys::libdeflate_gdeflate_compress(
                self.0,
                input.as_ptr() as *const std::ffi::c_void,
                input.len(),
                &mut compressed_page,
                1,
            )
        };

        if result == 0 {
            Err(CompressionError::CompressionFailed)?
        }

        let compressed_data = unsafe {
            std::slice::from_raw_parts(compressed_page.data as *const u8, compressed_page.nbytes)
                .to_vec()
        };

        Ok(compressed_data)
    }

    pub fn compress(&mut self, in_data: &[u8]) -> Result<Vec<u8>> {
        if in_data.is_empty() {
            return Err(CompressionError::CompressionFailed)?;
        }

        let num_items = in_data.len().div_ceil(KDEFAULT_TILE_SIZE) as u32;
        let mut tiles = Vec::with_capacity(num_items as usize);

        // 压缩每个tile
        for tile_index in 0..num_items {
            let tile_pos = tile_index as usize * KDEFAULT_TILE_SIZE;
            let remaining = in_data.len() - tile_pos;
            let uncompressed_size = std::cmp::min(remaining, KDEFAULT_TILE_SIZE);

            let tile_data = &in_data[tile_pos..tile_pos + uncompressed_size];
            let compressed_tile = self.compress_tile(tile_data)?;
            tiles.push(compressed_tile);
        }

        // 准备输出流
        let mut tile_ptrs = Vec::with_capacity(num_items as usize);
        let mut data_pos = 0;

        for tile in &tiles {
            tile_ptrs.push(data_pos as u32);
            data_pos += tile.len();
        }

        // tile_ptrs[0]用于存储最后一个tile的大小
        if !tile_ptrs.is_empty() {
            let last_tile_size = tiles.last().unwrap().len();
            tile_ptrs[0] = last_tile_size as u32;
        }

        // 计算未压缩大小
        let header = TileStream::new(in_data.len());

        // 组装输出数据
        let mut output = Vec::new();

        // 写入header
        output.extend_from_slice(&[header.id]);
        output.extend_from_slice(&[header.magic]);
        output.extend_from_slice(&header.num_tiles.to_le_bytes());

        let flags = (header.tile_size_idx & 0x3)
            | ((header.last_tile_size & 0x3FFFF) << 2)
            | ((header.reserv1 & 0xFFF) << 20);
        output.extend_from_slice(&flags.to_le_bytes());

        // 写入tile偏移表
        for ptr in &tile_ptrs {
            output.extend_from_slice(&ptr.to_le_bytes());
        }

        // 写入压缩数据
        for (i, tile) in tiles.iter().enumerate() {
            let tile_offset = if i == 0 { 0 } else { tile_ptrs[i] as usize };
            while output.len() < tile_offset {
                output.push(0);
            }
            output.extend_from_slice(tile);
        }

        Ok(output)
    }
}

impl Drop for GDfCompressor {
    fn drop(&mut self) {
        unsafe {
            sys::libdeflate_free_gdeflate_compressor(self.0);
        }
    }
}
