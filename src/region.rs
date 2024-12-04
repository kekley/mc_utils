use std::cell::RefCell;
use std::fs::File;
use std::io::{self, Cursor, Read, Seek};
use std::sync::Arc;
use std::{usize, vec};

use bytes::{Buf, Bytes};

use crate::chunk::Chunk;
use crate::compression::{CompressionData, CompressionScheme};
use crate::spider_eye_error::SpiderEyeError;
use crate::ChunkData;

//offsets are for 4KiB Sectors
pub(crate) const CHUNKS_PER_FILE: usize = 1024;
pub(crate) const SECTOR_SIZE: usize = 4096;

// Header consists of two 4KiB Tables
pub(crate) const REGION_HEADER_SIZE: usize = 2 * SECTOR_SIZE;

// The size of the header for a chunk which immediate proceeds the compressed chunk data
pub(crate) const CHUNK_HEADER_SIZE: usize = 5;

#[derive(Debug, Clone)]
pub struct Region {
    pub x: i32,
    pub z: i32,
    pub data: Bytes,
    pub compresssed_chunks: [Option<Bytes>; CHUNKS_PER_FILE],
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FileSegment {
    pub sector_offset: u32,
    pub sectors: u8,
}

impl FileSegment {
    pub fn new(offset: u32, sectors: u8) -> Self {
        Self {
            sector_offset: offset,
            sectors,
        }
    }
}

impl Region {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, SpiderEyeError> {
        const ARRAY_REPEAT_VALUE: Option<bytes::Bytes> = None;
        let data = Bytes::from(bytes);
        let mut region: Region = Self {
            data: data,
            compresssed_chunks: [ARRAY_REPEAT_VALUE; 1024],
            x: 0,
            z: 0,
        };

        let segments: [Option<FileSegment>; CHUNKS_PER_FILE] = region.read_chunk_segments()?;

        let compressed_chunks = segments
            .iter()
            .map(|opt| {
                if let Some(segment) = opt {
                    let compressed_chunk = region.get_compressed_chunk(segment);
                    Some(compressed_chunk)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        region.compresssed_chunks = compressed_chunks
            .try_into()
            .expect("compressed chunks array of wrong size");
        Ok(region)
    }

    fn get_compressed_chunk(&self, segment: &FileSegment) -> Bytes {
        let offset = segment.sector_offset as usize * SECTOR_SIZE;

        let len = segment.sectors as usize * SECTOR_SIZE;
        let compressed_bytes = self.data.slice(offset..offset + len);

        compressed_bytes
    }

    fn read_chunk_segments(&mut self) -> Result<[Option<FileSegment>; 1024], SpiderEyeError> {
        let mut vec: Vec<Option<FileSegment>> = Vec::with_capacity(1024);

        (0..32).for_each(|x| {
            (0..32).for_each(|z| {
                let offset = Self::get_segment_pos(x, z);
                let segment_bytes = self
                    .data
                    .get((offset..offset + 4))
                    .expect("ran out of bytes reading chunk segments???");
                let offset: u32 = ((segment_bytes[0] as u32) << 16)
                    | ((segment_bytes[1] as u32) << 8)
                    | (segment_bytes[2] as u32);

                let sectors: u8 = segment_bytes[3];

                if offset == 0 || sectors == 0 {
                    vec.push(None);
                } else {
                    vec.push(Some(FileSegment::new(offset, sectors)));
                }
            });
        });

        vec.try_into().map_err(|_| SpiderEyeError::InvalidFile())
    }

    pub fn get_chunk(&self, x: usize, z: usize) -> Option<Chunk> {
        if x > 32 || z > 32 {
            return None;
        }
        if let Some(chunk_data) = &self.compresssed_chunks[x * 32 + z] {
            let decompressed = Self::decompress_chunk(chunk_data);
            let chunk = Chunk::from_bytes(decompressed).expect("Invalid chunk data");
            Some(chunk)
        } else {
            None
        }
    }

    pub fn decompress_chunk(data: &Bytes) -> Bytes {
        let compression_data = Self::get_compression_data(data).unwrap();
        let compressed_data = data
            .get(5..5 + compression_data.compressed_len as usize)
            .expect("ran out of bytes reading compressed data");
        let mut cursor = Cursor::new(compressed_data);
        let res = match compression_data.scheme {
            CompressionScheme::Gzip => {
                let mut writer = flate2::write::GzDecoder::new(Vec::with_capacity(1000));
                io::copy(&mut cursor, &mut writer).unwrap();
                writer.finish().unwrap()
            }
            CompressionScheme::Zlib => {
                let mut writer = flate2::write::ZlibDecoder::new(Vec::with_capacity(1000));
                io::copy(&mut cursor, &mut writer).unwrap();
                writer.finish().unwrap()
            }
            CompressionScheme::Uncompressed => {
                let mut writer = Vec::with_capacity(1000);
                io::copy(&mut cursor, &mut writer).unwrap();
                writer
            }
        };

        Bytes::from(res)
    }

    fn get_compression_data(data: &Bytes) -> Result<CompressionData, SpiderEyeError> {
        let chunk_header = data
            .get((0..5))
            .expect("ran out of bytes getting compression data");

        let compression_data = CompressionData::new(&chunk_header)?;

        Ok(compression_data)
    }

    fn get_segment_pos(x: usize, z: usize) -> usize {
        4 * ((x & 31) + ((z & 31) << 5))
    }
}
