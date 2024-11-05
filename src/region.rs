use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use std::io::{self, Error, Read, Seek, Write};
use std::time::SystemTime;
use std::{usize, vec};

use crate::nbt_ids::NBTId;

//offsets are for 4KiB Sectors
pub(crate) const CHUNKS_PER_FILE: usize = 1024;
pub(crate) const SECTOR_SIZE: usize = 4096;

// Header consists of two 4KiB Tables
pub(crate) const REGION_HEADER_SIZE: usize = 2 * SECTOR_SIZE;

// The size of the header for a chunk which immediate proceeds the compressed chunk data
pub(crate) const CHUNK_HEADER_SIZE: usize = 5;

#[derive(Debug)]
pub enum RegionError {
    DEFAULT,
    IO(std::io::Error),
    InvalidOffset(isize, isize),
    UnknownCompression(u8),
    TryFromPrimitiveError(TryFromPrimitiveError<NBTId>),
    ListError(i32),
    UTF8Error(std::string::FromUtf8Error),
    JavaStringDecodingError(cesu8::Cesu8DecodingError),
}

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum CompressionScheme {
    Gzip = 1,
    Zlib = 2,
    Uncompressed = 3,
}

impl From<std::io::Error> for RegionError {
    fn from(value: std::io::Error) -> Self {
        RegionError::IO(value)
    }
}

impl From<num_enum::TryFromPrimitiveError<NBTId>> for RegionError {
    fn from(value: num_enum::TryFromPrimitiveError<NBTId>) -> Self {
        RegionError::TryFromPrimitiveError(value)
    }
}

impl From<std::string::FromUtf8Error> for RegionError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        RegionError::UTF8Error(value)
    }
}

impl From<cesu8::Cesu8DecodingError> for RegionError {
    fn from(value: cesu8::Cesu8DecodingError) -> Self {
        RegionError::JavaStringDecodingError(value)
    }
}

impl std::fmt::Display for RegionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegionError::DEFAULT => todo!(),
            RegionError::IO(error) => f.write_fmt(format_args!("IO Error: {error:?}")),
            RegionError::InvalidOffset(x, z) => {
                f.write_fmt(format_args!("Invalid Offset: x: {x}, z: {z}"))
            }
            RegionError::UnknownCompression(val) => {
                f.write_fmt(format_args!("Unknown Compresssion sceme: {val}"))
            }
            RegionError::TryFromPrimitiveError(try_from_primitive_error) => f.write_fmt(
                format_args!("Failed Primitive Conversion: {try_from_primitive_error:?}"),
            ),
            RegionError::UTF8Error(utf_8_error) => f.write_fmt(format_args!(
                "Error creating string from bytes: {utf_8_error:?}"
            )),
            RegionError::JavaStringDecodingError(cesu8_decoding_error) => f.write_fmt(
                format_args!("Error parsing a java string: {cesu8_decoding_error:?}"),
            ),
            RegionError::ListError(len) => {
                f.write_fmt(format_args!("Error parsing list of len {len}"))
            }
        }
    }
}

impl std::error::Error for RegionError {}

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

#[derive(Debug, Clone)]
pub struct Chunk {
    world_location: (u64, u64),
    timestamp: SystemTime,
    chunk_data: ChunkData,
}

pub struct CompressionData {
    pub scheme: CompressionScheme,
    pub compressed_len: u32,
}

impl CompressionData {
    pub fn new(mut data: &[u8]) -> Result<Self, RegionError> {
        println!("{:?}", &data[..]);

        let len = data.read_u32::<BigEndian>()?;
        let scheme = data.read_u8()?;
        let compression_data = Self {
            scheme: CompressionScheme::try_from(scheme)
                .map_err(|_| RegionError::UnknownCompression(scheme))?,
            compressed_len: len - 1,
        };

        Ok(compression_data)
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            world_location: Default::default(),
            timestamp: SystemTime::UNIX_EPOCH,
            chunk_data: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ChunkData {}

impl<S> Region<S>
where
    S: Read + Seek,
{
    pub fn from_stream(mut stream: S) -> Result<Self, RegionError> {
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
    ) -> Result<Option<FileSegment>, RegionError> {
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
    ) -> Result<Vec<u8>, RegionError> {
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
    ) -> Result<CompressionData, RegionError> {
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
