use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use std::io::{self, Error, Read, Seek, Write};
use std::time::SystemTime;
use std::{usize, vec};

use crate::compression::{CompressionData, CompressionScheme};
use crate::nbt_ids::NBTId;
use crate::spider_eye_error::SpiderEyeError;

//offsets are for 4KiB Sectors
pub(crate) const CHUNKS_PER_FILE: usize = 1024;
pub(crate) const SECTOR_SIZE: usize = 4096;

// Header consists of two 4KiB Tables
pub(crate) const REGION_HEADER_SIZE: usize = 2 * SECTOR_SIZE;

// The size of the header for a chunk which immediate proceeds the compressed chunk data
pub(crate) const CHUNK_HEADER_SIZE: usize = 5;

#[derive(Debug, Default)]
pub struct Region<S> {
    pub stream: S,
    pub chunk_segments: Vec<FileSegment>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FileSegment {
    pub offset: u32,
    pub sectors: u8,
}

impl FileSegment {
    pub fn new(offset: u32, sectors: u8) -> Self {
        Self { offset, sectors }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ChunkData {}

impl<S> Region<S>
where
    S: Read + Seek,
{
    pub fn from_stream(mut stream: S) -> Result<Self, SpiderEyeError> {
        let mut region = Self {
            stream,
            chunk_segments: vec![],
        };
        let mut chunk_segments: Vec<FileSegment> = Vec::with_capacity(CHUNKS_PER_FILE);
        for x in 0..32 {
            for z in 0..32 {
                let segment = region.get_chunk_segment(x, z)?;
                match segment {
                    Some(chunk_segment) => {
                        chunk_segments.push(chunk_segment);
                    }
                    None => {
                        continue;
                    }
                }
            }
        }

        region.chunk_segments = chunk_segments;
        Ok(region)
    }

    pub fn get_chunk_segment(
        &mut self,
        x: usize,
        z: usize,
    ) -> Result<Option<FileSegment>, SpiderEyeError> {
        self.stream.seek(std::io::SeekFrom::Start(
            Self::get_header_pos_in_stream(x, z).try_into().unwrap(),
        ))?;
        let mut buf: [u8; 4] = [0; 4];
        self.stream.read_exact(&mut buf)?;
        let offset: u32 = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);
        let sectors: u8 = buf[3];

        if offset == 0 || sectors == 0 {
            Ok(None)
        } else {
            Ok(Some(FileSegment::new(offset, sectors)))
        }
    }

    pub fn read_chunk_from_segment(
        &mut self,
        segment: FileSegment,
    ) -> Result<Vec<u8>, SpiderEyeError> {
        let compression_data = self.get_compression_data(segment)?;

        let mut take = (&mut self.stream).take(compression_data.compressed_len as u64);

        match compression_data.scheme {
            CompressionScheme::Gzip => {
                let mut writer = flate2::write::GzDecoder::new(vec![]);
                io::copy(&mut take, &mut writer)?;
                Ok(writer.finish()?)
            }
            CompressionScheme::Zlib => {
                let a = 0;
                let mut writer = flate2::write::ZlibDecoder::new(vec![]);
                io::copy(&mut take, &mut writer)?;
                Ok(writer.finish()?)
            }
            CompressionScheme::Uncompressed => {
                let mut writer = vec![];
                io::copy(&mut take, &mut writer)?;
                Ok(writer)
            }
        }
    }

    pub fn get_compression_data(
        &mut self,
        segment: FileSegment,
    ) -> Result<CompressionData, SpiderEyeError> {
        self.stream.seek(std::io::SeekFrom::Start(
            segment.offset as u64 * SECTOR_SIZE as u64,
        ))?;

        let mut buff: [u8; 5] = [0; 5];
        self.stream.read_exact(&mut buff)?;

        let compression_data = CompressionData::new(&buff)?;

        Ok(compression_data)
    }

    pub fn get_header_pos_in_stream(x: usize, z: usize) -> usize {
        4 * ((x & 32) + (z & 32) * 32)
    }
}
