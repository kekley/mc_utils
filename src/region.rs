use std::cell::RefCell;
use std::fs::File;
use std::io::{self, BufReader, Cursor, Read, Seek};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::{usize, vec};

use byteorder::ReadBytesExt;
use bytes::{Buf, Bytes};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

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
#[derive(Debug)]
pub struct Region {
    pub x: i32,
    pub z: i32,
    pub file_path: String,
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
    pub fn from_file(path: &str) -> Result<Self, SpiderEyeError> {
        let mut region: Region = Self {
            file_path: path.to_string(),
            x: 0,
            z: 0,
        };

        let a = (0..32).any(|z| {
            (0..32).any(|x| {
                if let Some(chunk) = region.get_chunk(x, z) {
                    region.x = chunk.xpos >> 5;
                    region.z = chunk.zpos >> 5;
                    true
                } else {
                    false
                }
            })
        });

        if a {
            Ok(region)
        } else {
            Err(SpiderEyeError::InvalidFile())
        }
    }

    pub fn get_compressed_chunk(&self, segment: &FileSegment) -> Vec<u8> {
        let mut reader =
            BufReader::new(File::open(&self.file_path).expect("not a valid file path"));
        let offset = segment.sector_offset * SECTOR_SIZE as u32;
        let len = segment.sectors as usize * SECTOR_SIZE;
        let mut buf = vec![0u8; len];
        let _ = reader.seek(io::SeekFrom::Start((offset).into()));
        let _ = reader.read(&mut buf);

        buf
    }

    fn read_chunk_segment(&self, x: u32, z: u32, reader: &mut BufReader<File>) -> FileSegment {
        let offset = Self::get_segment_pos(x, z);
        let _ = reader.seek(io::SeekFrom::Start(offset as u64));

        let mut buf = [0u8; 4];
        let _ = reader.read_exact(&mut buf);

        let offset: u32 = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);

        let sectors: u8 = buf[3];

        FileSegment::new(offset, sectors)
    }

    pub fn get_chunk(&self, x: u32, z: u32) -> Option<Chunk> {
        if x > 32 || z > 32 {
            return None;
        }
        let mut reader =
            BufReader::new(File::open(&self.file_path).expect("not a valid file path"));

        let segment = self.read_chunk_segment(x, z, &mut reader);

        if segment.sector_offset == 0 || segment.sectors == 0 {
            return None;
        }
        let compressed_chunk = self.get_compressed_chunk(&segment);

        let decompressed_chunk = Self::decompress_chunk(&compressed_chunk);

        Some(Chunk::from_bytes(decompressed_chunk))
    }

    fn decompress_chunk(data: &Vec<u8>) -> Vec<u8> {
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

        res
    }

    fn get_compression_data(data: &Vec<u8>) -> Result<CompressionData, SpiderEyeError> {
        let chunk_header = data
            .get((0..5))
            .expect("ran out of bytes getting compression data");

        let compression_data = CompressionData::new(&chunk_header)?;

        Ok(compression_data)
    }

    fn get_segment_pos(x: u32, z: u32) -> u32 {
        4 * ((x & 31) + ((z & 31) << 5))
    }
}
