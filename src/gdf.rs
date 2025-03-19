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
        let mut bytes_read = 0;

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
                    sys::libdeflate_result_LIBDEFLATE_SUCCESS => bytes_read += out_nbytes,
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

    pub fn compress(&mut self, in_data: &[u8]) -> Result<Vec<u8>> {
        unimplemented!()
    }
}

impl Drop for GDfCompressor {
    fn drop(&mut self) {
        unsafe {
            sys::libdeflate_free_gdeflate_compressor(self.0);
        }
    }
}
