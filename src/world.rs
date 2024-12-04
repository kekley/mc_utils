use std::{
    collections::HashMap,
    fs,
    io::{Cursor, Read, Seek},
    path::Path,
};

use smol_str::SmolStr;

use crate::{Chunk, ChunkData, Region};

pub struct World {
    pub regions: HashMap<(i32, i32), Region>,
    pub loaded_chunks: HashMap<(i32, i32), ChunkData>,
    pub global_palette: Vec<SmolStr>,
}

impl World {
    pub fn new(folder_path: &str) -> Self {
        let mut temp = Self {
            regions: HashMap::new(),
            loaded_chunks: HashMap::new(),
            global_palette: vec![],
        };
        let read_dir = fs::read_dir(folder_path).expect("could not find folder");
        for dir in read_dir {
            let entry = dir.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension().unwrap() == "mca" {
                if let Ok(region) = Region::from_file(path.to_str().unwrap()) {
                    temp.regions.insert((region.x, region.z), region);
                }
            }
        }

        temp
    }

    pub fn get_region(&self, x: i32, z: i32) -> Option<&Region> {
        self.regions.get(&(x, z))
    }

    pub fn get_chunk(&self, x: i32, z: i32) -> Option<Chunk> {
        let opt = self.get_region_containing_chunk(x, z);
        let local_x = (x.abs() % 32) as u32;
        let local_z = (z.abs() % 32) as u32;
        if let Some(region) = opt {
            region.get_chunk(local_x, local_z)
        } else {
            None
        }
    }

    fn get_region_containing_chunk(&self, x: i32, z: i32) -> Option<&Region> {
        self.get_region(x >> 5, z >> 5)
    }

    fn get_compressed_chunk(&self, x: i32, z: i32) -> Vec<u8> {
        todo!()
    }
    fn get_region_containing_block(&self, x: i64, z: i64) -> Option<&Region> {
        self.get_region_containing_chunk((x >> 4) as i32, (z >> 4) as i32)
    }
}
