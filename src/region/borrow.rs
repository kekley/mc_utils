use std::{error::Error, path::Path, rc::Rc};

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    compression::decompress_chunk, coords::region::RegionCoords,
    error::spider_eye_error::SpiderEyeError,
};

const SECTOR_SIZE: usize = 4096;

pub struct Region<'a> {
    x: i64,
    z: i64,
    data: &'a [u8],
}

impl<'a> Region<'a> {
    pub fn from_bytes(data: &'a [u8], coords: RegionCoords) -> Region<'a> {
        Region {
            x: coords.x,
            z: coords.z,
            data,
        }
    }

    pub fn get_region_x(&self) -> i64 {
        self.x
    }

    pub fn get_region_z(&self) -> i64 {
        self.z
    }

    fn get_chunk_header_offset(x: u8, z: u8) -> usize {
        4 * ((x as usize & 31) + ((z as usize & 31) << 5))
    }

    pub fn load_chunk_data(&self, x: u8, z: u8) -> Option<Box<[u8]>> {
        let header_offset = Self::get_chunk_header_offset(x, z);

        let buf: [u8; 4] = self
            .data
            .get(header_offset..header_offset + 4)?
            .try_into()
            .expect("Slice should be three bytes long");
        let offset = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);

        let sectors = buf[3];

        let chunk_offset_in_file = offset as usize * SECTOR_SIZE;

        let chunk_length_in_bytes = sectors as usize * SECTOR_SIZE;

        let chunk_data_slice = self
            .data
            .get(chunk_offset_in_file..chunk_offset_in_file + chunk_length_in_bytes)?;

        let decompressed_bytes = decompress_chunk(chunk_data_slice).ok()?;

        Some(decompressed_bytes.into_boxed_slice())
    }
    pub fn load_all_chunk_data(&self) -> [Option<Box<[u8]>>; 1024] {
        let a = (0..32)
            .into_par_iter()
            .flat_map(move |z| {
                (0..32)
                    .into_par_iter()
                    .map(move |x| self.load_chunk_data(x as u8, z as u8))
            })
            .collect::<Vec<_>>();

        a.try_into().unwrap()
    }
}
