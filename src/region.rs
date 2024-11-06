use std::io::{self, Cursor, Read, Seek};
use std::{usize, vec};

use crate::chunk::Chunk;
use crate::compression::{CompressionData, CompressionScheme};
use crate::spider_eye_error::SpiderEyeError;

//offsets are for 4KiB Sectors
pub(crate) const CHUNKS_PER_FILE: usize = 1024;
pub(crate) const SECTOR_SIZE: usize = 4096;

// Header consists of two 4KiB Tables
pub(crate) const REGION_HEADER_SIZE: usize = 2 * SECTOR_SIZE;

// The size of the header for a chunk which immediate proceeds the compressed chunk data
pub(crate) const CHUNK_HEADER_SIZE: usize = 5;

#[derive(Debug)]
pub struct Region<S> {
    pub stream: S,
    pub chunk_segments: [Option<FileSegment>; CHUNKS_PER_FILE],
}

impl<S: Default> Default for Region<S> {
    fn default() -> Self {
        Self {
            stream: Default::default(),
            chunk_segments: [None; 1024],
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

impl<S> Region<S>
where
    S: Read + Seek,
{
    pub fn from_stream(mut stream: S) -> Result<Self, SpiderEyeError> {
        let mut region: Region<S> = Self {
            stream: stream,
            chunk_segments: [None; 1024],
        };
        for x in 0..32 {
            for z in 0..32 {
                let segment = region.read_chunk_segment(x, z)?;
                region.chunk_segments[x * 32 + z] = segment;
            }
        }

        Ok(region)
    }

    fn read_chunk_segment(
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

    pub fn get_chunk_segment(&mut self, x: usize, z: usize) -> Option<FileSegment> {
        self.chunk_segments[x * 32 + z]
    }
    pub fn get_chunk(&mut self, x: usize, z: usize) -> Option<Chunk> {
        if x > 32 || z > 32 {
            return None;
        }
        if let Some(segment) = self.get_chunk_segment(x, z) {
            let data = self
                .read_chunk_from_segment(segment)
                .expect("Failed to read chunk. Likely invalid file");
            let mut cursor = Cursor::new(data);
            Some(Chunk::from_data(&mut cursor).expect("Failed to parse chunk data"))
        } else {
            None
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

    fn get_header_pos_in_stream(x: usize, z: usize) -> usize {
        4 * ((x & 32) + (z & 32) * 32)
    }
}
