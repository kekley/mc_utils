use std::fs::File;
use std::io::{self, BufReader, Cursor, Read, Seek};
use std::sync::{Arc, RwLock};
use std::{usize, vec};

use crate::block::Block;
use crate::block_states::InternalBlockState;
use crate::chunk::Chunk;

use crate::nbt::compression::{CompressionData, CompressionScheme};
use crate::nbt_compound::NBTCompound;
use crate::palette::BlockPalette;
use crate::spider_eye_error::SpiderEyeError;
use crate::variant::BlockName;
use crate::ResourceLoader;
use bytes::Bytes;
use fxhash::FxBuildHasher;
use lasso::{Spur, ThreadedRodeo};
use smol_str::SmolStr;

use super::loaded_world::RegionCoords;

//offsets are for 4KiB Sectors
pub(crate) const CHUNKS_PER_FILE: usize = 1024;
pub(crate) const SECTOR_SIZE: usize = 4096;

// Header consists of two 4KiB Tables
pub(crate) const REGION_HEADER_SIZE: usize = 2 * SECTOR_SIZE;

// The size of the header for a chunk which immediate proceeds the compressed chunk data
pub(crate) const CHUNK_HEADER_SIZE: usize = 5;

#[derive(Debug, Clone)]
pub struct LoadedRegion {
    pub coords: RegionCoords,
    chunks: Box<[Option<Chunk>; 1024]>,
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

impl LoadedRegion {
    fn load_region(path: String, resource_loader: &ResourceLoader) -> Result<Self, SpiderEyeError> {
        let mut chunks: Box<[Option<Chunk>; 1024]> = Box::new([const { None }; 1024]);
        let mut reader =
            BufReader::new(File::open(&path).expect("not a valid file path for region"));
        for z in 0..32 {
            for x in 0..32 {
                let segment = Self::read_chunk_segment(x, z, &mut reader);
                if segment.sector_offset != 0 && segment.sectors != 0 {
                    let compressed_chunk_data = Self::get_compressed_chunk(&mut reader, &segment);
                    let chunk_bytes = Self::decompress_chunk(&compressed_chunk_data);
                    let mut bytes = Bytes::from(chunk_bytes);
                    let chunk_nbt = resource_loader
                        .nbt_from_bytes(&mut bytes)
                        .expect("Chunk NBT was invalid");
                    Chunk::from_nbt(nbt_compound, palette, rodeo)
                }
            }
        }
        unimplemented!()
    }

    fn get_compressed_chunk(reader: &mut BufReader<File>, segment: &FileSegment) -> Vec<u8> {
        let offset = segment.sector_offset * SECTOR_SIZE as u32;
        let len = segment.sectors as usize * SECTOR_SIZE;
        let mut buf = vec![0u8; len];
        let _ = reader.seek(io::SeekFrom::Start((offset).into()));
        let _ = reader.read(&mut buf);

        buf
    }

    fn read_chunk_segment(x: u32, z: u32, reader: &mut BufReader<File>) -> FileSegment {
        let offset = Self::get_segment_pos(x, z);
        let _ = reader.seek(io::SeekFrom::Start(offset as u64));

        let mut buf = [0u8; 4];
        let _ = reader.read_exact(&mut buf);

        let offset: u32 = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);

        let sectors: u8 = buf[3];

        FileSegment::new(offset, sectors)
    }

    pub fn get_chunk(&self, x: u32, z: u32, palette: &BlockPalette<Block>) -> Option<Chunk> {
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

        let mut bytes = Bytes::from(decompressed_chunk);
        let nbt = self
            .resource_loader
            .nbt_from_bytes(&mut bytes)
            .expect("invalid nbt");
        Some(Chunk::from_nbt(nbt, palette, &self.resource_loader.rodeo))
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

    pub fn get_all_chunks(&self, palette: &BlockPalette<Block>) -> Vec<Chunk> {
        let mut chunks = vec![];
        (0..32).for_each(|z| {
            (0..32).for_each(|x| {
                let opt = self.get_chunk(x, z, &palette);
                if opt.is_some() {
                    chunks.push(opt.unwrap());
                }
            });
        });
        chunks
    }

    fn get_compression_data(data: &Vec<u8>) -> Result<CompressionData, SpiderEyeError> {
        let chunk_header = data
            .get(0..5)
            .expect("ran out of bytes getting compression data");

        let compression_data = CompressionData::new(&chunk_header)?;

        Ok(compression_data)
    }

    fn get_segment_pos(x: u32, z: u32) -> u32 {
        4 * ((x & 31) + ((z & 31) << 5))
    }
}
