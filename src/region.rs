use std::cell::RefCell;
use std::io::{self, Cursor, Read, Seek};
use std::{usize, vec};

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
    pub data: RefCell<Cursor<Vec<u8>>>,
    pub chunk_segments: [Option<FileSegment>; CHUNKS_PER_FILE],
}

impl Default for Region {
    fn default() -> Self {
        Self {
            data: Default::default(),
            chunk_segments: [None; 1024],
            x: 0,
            z: 0,
        }
    }
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

impl Region {
    pub fn from_stream(stream: Cursor<Vec<u8>>) -> Result<Self, SpiderEyeError> {
        let mut region: Region = Self {
            data: RefCell::new(stream),
            chunk_segments: [None; 1024],
            x: 0,
            z: 0,
        };
        for x in 0..32 {
            for z in 0..32 {
                let segment = region.read_chunk_segment(x, z)?;
                region.chunk_segments[x * 32 + z] = segment;
            }
        }

        region.chunk_segments.into_iter().any(|f| {
            if let Some(_segment) = f {
                let bytes = region.read_chunk_from_segment(_segment);
                let chunk = Chunk::from_slice(&bytes).unwrap();
                region.x = chunk.xpos >> 4;
                region.z = chunk.zpos >> 4;
                true
            } else {
                false
            }
        });

        Ok(region)
    }

    fn read_chunk_segment(
        &mut self,
        x: usize,
        z: usize,
    ) -> Result<Option<FileSegment>, SpiderEyeError> {
        let pos = Self::get_header_pos_in_stream(x, z);
        self.data.borrow_mut().seek(std::io::SeekFrom::Start(
            Self::get_header_pos_in_stream(x, z) as u64,
        ))?;
        let mut buf: [u8; 4] = [0; 4];
        self.data.borrow_mut().read_exact(&mut buf[..])?;
        let offset: u32 = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);
        let sectors: u8 = buf[3];

        if offset == 0 || sectors == 0 {
            Ok(None)
        } else {
            Ok(Some(FileSegment::new(offset, sectors)))
        }
    }

    pub fn get_chunk_segment(&self, x: usize, z: usize) -> Option<FileSegment> {
        self.chunk_segments[x * 32 + z]
    }

    pub fn get_chunk(&mut self, x: usize, z: usize) -> Option<Chunk> {
        if x > 32 || z > 32 {
            return None;
        }
        if let Some(segment) = self.get_chunk_segment(x, z) {
            let data = self.read_chunk_from_segment(segment);
            Some(Chunk::from_slice(&data).expect("Failed to parse chunk data"))
        } else {
            None
        }
    }

    pub fn read_chunk_from_segment(&self, segment: FileSegment) -> Vec<u8> {
        let compression_data = self.get_compression_data(segment).unwrap();

        let mut buf = vec![0u8; compression_data.compressed_len as usize];
        self.data.borrow_mut().read_exact(&mut buf[..]).unwrap();
        let mut cursor = Cursor::new(buf);

        match compression_data.scheme {
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
        }
    }

    pub fn get_compression_data(
        &self,
        segment: FileSegment,
    ) -> Result<CompressionData, SpiderEyeError> {
        self.data.borrow_mut().seek(std::io::SeekFrom::Start(
            segment.offset as u64 * SECTOR_SIZE as u64,
        ))?;

        let mut buff: [u8; 5] = [0; 5];
        self.data.borrow_mut().read_exact(&mut buff)?;

        let compression_data = CompressionData::new(&buff)?;

        Ok(compression_data)
    }

    fn get_header_pos_in_stream(x: usize, z: usize) -> usize {
        4 * ((x & 31) + ((z & 31) << 5))
    }
}
