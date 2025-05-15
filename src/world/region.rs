use std::fs::File;
use std::io::{self, BufReader, Cursor, Read, Seek};
use std::sync::Arc;
use std::{usize, vec};

use crate::chunk::Chunk;

use crate::nbt::compression::{CompressionData, CompressionScheme};
use crate::nbt_compound::NBTCompound;

use anyhow::{anyhow, Context, Ok};
use bytes::Bytes;
use lasso::ThreadedRodeo;

use super::loaded_world::RegionCoords;

//offsets are for 4KiB Sectors
pub(crate) const CHUNKS_PER_FILE: usize = 1024;
pub(crate) const SECTOR_SIZE: usize = 4096;

// Header consists of two 4KiB Tables
pub(crate) const REGION_HEADER_SIZE: usize = 2 * SECTOR_SIZE;

// The size of the header for a chunk which immediate proceeds the compressed chunk data
pub(crate) const CHUNK_HEADER_SIZE: usize = 5;

#[derive(Debug)]
pub struct LazyRegion {
    interner: Arc<ThreadedRodeo>,
    pub file_path: String,
    pub coords: RegionCoords,
    segments: [FileSegment; 1024],
}

impl LazyRegion {
    pub fn new(path: &str, interner: &Arc<ThreadedRodeo>) -> anyhow::Result<Self> {
        let mut segments: [FileSegment; 1024] = [const {
            FileSegment {
                sector_offset: 0,
                sectors: 0,
            }
        }; 1024];

        let mut reader =
            BufReader::new(File::open(&path).context("Region file {path} does not exist")?);
        for z in 0..32 {
            for x in 0..32 {
                let segment = read_chunk_segment(x, z, &mut reader);
                segments[(x + z * 32) as usize] = segment
            }
        }

        let first_valid_segment = segments
            .iter()
            .find(|segment| segment.sector_offset != 0 && segment.sectors != 0);

        let coords = match first_valid_segment {
            Some(segment) => {
                let compressed = LoadedRegion::get_compressed_chunk(&mut reader, segment);
                let chunk = LoadedRegion::decompress_chunk(&compressed);
                let mut bytes = Bytes::from(chunk);
                let nbt = NBTCompound::internal_nbt(&mut bytes, interner)?;
                let chunk = Chunk::from_nbt_internal(nbt, interner);
                let coords = RegionCoords::from(chunk.coords);
                coords
            }
            None => return Err(anyhow!("No valid chunk segments")),
        };

        let value = LazyRegion {
            interner: interner.clone(),
            file_path: path.to_owned(),
            coords,
            segments: segments,
        };
        Ok(value)
    }
}

#[derive(Debug, Clone)]
pub struct LoadedRegion {
    interner: Arc<ThreadedRodeo>,
    pub coords: RegionCoords,
    chunks: Box<[Option<Chunk>; 1024]>,
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

impl From<&LazyRegion> for LoadedRegion {
    fn from(value: &LazyRegion) -> Self {
        let LazyRegion {
            interner,
            file_path,
            coords,
            segments,
        } = value;
        let mut chunks: Box<[Option<Chunk>; 1024]> = Box::new([const { None }; 1024]);
        let mut reader = BufReader::new(File::open(file_path).unwrap());
        for z in 0..32 {
            for x in 0..32 {
                let segment = segments[x + 32 * z];
                if segment.sector_offset != 0 && segment.sectors != 0 {
                    let compressed_chunk_data = Self::get_compressed_chunk(&mut reader, &segment);
                    let chunk_bytes = Self::decompress_chunk(&compressed_chunk_data);
                    let mut bytes = Bytes::from(chunk_bytes);
                    let chunk_nbt = NBTCompound::internal_nbt(&mut bytes, &interner)
                        .expect("Chunk NBT was invalid");
                    let chunk = Chunk::from_nbt_internal(chunk_nbt, &interner);
                    chunks[(x + z * 32) as usize] = Some(chunk);
                }
            }
        }

        LoadedRegion {
            interner: interner.clone(),
            coords: coords.clone(),
            chunks: chunks,
        }
    }
}

impl LoadedRegion {
    pub(crate) fn load_region(
        path: &str,
        interner: &Arc<ThreadedRodeo>,
    ) -> anyhow::Result<LoadedRegion> {
        let mut chunks: Box<[Option<Chunk>; 1024]> = Box::new([const { None }; 1024]);
        let mut reader =
            BufReader::new(File::open(&path).expect("not a valid file path for region"));
        for z in 0..32 {
            for x in 0..32 {
                let segment = read_chunk_segment(x, z, &mut reader);
                if segment.sector_offset != 0 && segment.sectors != 0 {
                    let compressed_chunk_data = Self::get_compressed_chunk(&mut reader, &segment);
                    let chunk_bytes = Self::decompress_chunk(&compressed_chunk_data);
                    let mut bytes = Bytes::from(chunk_bytes);
                    let chunk_nbt = NBTCompound::internal_nbt(&mut bytes, interner)
                        .expect("Chunk NBT was invalid");
                    let chunk = Chunk::from_nbt_internal(chunk_nbt, interner);
                    chunks[(x + z * 32) as usize] = Some(chunk);
                }
            }
        }

        Ok(LoadedRegion {
            interner: interner.clone(),
            coords: todo!(),
            chunks,
        })
    }

    fn get_compressed_chunk(reader: &mut BufReader<File>, segment: &FileSegment) -> Vec<u8> {
        let offset = segment.sector_offset * SECTOR_SIZE as u32;
        let len = segment.sectors as usize * SECTOR_SIZE;
        let mut buf = vec![0u8; len];
        let _ = reader.seek(io::SeekFrom::Start((offset).into()));
        let _ = reader.read(&mut buf);

        buf
    }

    pub fn get_chunk(&self, x: u32, z: u32) -> Option<&Chunk> {
        if x > 32 || z > 32 {
            return None;
        }
        self.chunks[(x + z * 32) as usize].as_ref()
    }

    pub fn get_all_chunks(self) -> Box<[Option<Chunk>; 1024]> {
        self.chunks
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

    fn get_compression_data(data: &Vec<u8>) -> anyhow::Result<CompressionData> {
        let chunk_header = data
            .get(0..5)
            .expect("ran out of bytes getting compression data");

        let compression_data = CompressionData::new(&chunk_header)?;

        Ok(compression_data)
    }
}

fn read_chunk_segment(x: u32, z: u32, reader: &mut BufReader<File>) -> FileSegment {
    let offset = get_segment_pos(x, z);
    let _ = reader.seek(io::SeekFrom::Start(offset as u64));

    let mut buf = [0u8; 4];
    let _ = reader.read_exact(&mut buf);

    let offset: u32 = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);

    let sectors: u8 = buf[3];

    FileSegment::new(offset, sectors)
}
fn get_segment_pos(x: u32, z: u32) -> u32 {
    4 * ((x & 31) + ((z & 31) << 5))
}
