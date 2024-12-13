use std::{
    collections::HashMap,
    fs,
    sync::{Arc, RwLock},
};

use indexmap::IndexMap;

use crate::{Chunk, Region};
#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub struct WorldCoords {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}
#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]

pub struct ChunkCoords {
    pub x: i64,
    pub z: i64,
}

impl ChunkCoords {
    pub fn new(x: i64, z: i64) -> Self {
        Self { x: x, z: z }
    }
}
#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]

pub struct RegionCoords {
    pub x: i64,
    pub z: i64,
}

impl From<ChunkCoords> for RegionCoords {
    fn from(value: ChunkCoords) -> Self {
        Self {
            x: value.x >> 5,
            z: value.z >> 5,
        }
    }
}

impl From<WorldCoords> for RegionCoords {
    fn from(value: WorldCoords) -> Self {
        Self {
            x: value.x >> 9,
            z: value.z >> 9,
        }
    }
}

impl From<WorldCoords> for ChunkCoords {
    fn from(value: WorldCoords) -> Self {
        Self {
            x: value.x >> 4,
            z: value.z >> 4,
        }
    }
}

impl From<ChunkCoords> for WorldCoords {
    fn from(value: ChunkCoords) -> Self {
        Self {
            x: 16 * value.x,
            y: 0,
            z: 16 * value.z,
        }
    }
}

#[derive(Debug)]
pub struct World {
    pub regions: HashMap<RegionCoords, Region>,
    pub loaded_chunks: HashMap<ChunkCoords, Chunk>,
    pub global_palette: Arc<RwLock<IndexMap<String, ()>>>,
}

impl World {
    pub fn new(folder_path: &str) -> Self {
        let mut palette = IndexMap::with_capacity(1);
        palette.insert_full("minecraft:air".to_string(), ());
        let mut temp = Self {
            regions: HashMap::new(),
            loaded_chunks: HashMap::new(),
            global_palette: Arc::new(RwLock::new(palette)),
        };
        let read_dir = fs::read_dir(folder_path).expect("could not find folder");
        for dir in read_dir {
            let entry = dir.unwrap();

            let path = entry.path();
            if path.is_file() && path.extension().unwrap() == "mca" {
                if let Ok(region) = Region::from_file(
                    path.to_str().unwrap().to_owned(),
                    temp.global_palette.clone(),
                ) {
                    temp.regions.insert(region.coords, region);
                }
            }
        }
        temp
    }

    pub fn get_region(&self, region_coords: RegionCoords) -> Option<&Region> {
        self.regions.get(&region_coords)
    }
    fn modulo(a: i64, b: i64) -> i64 {
        let r = a % b;
        if r < 0 {
            r + b
        } else {
            r
        }
    }

    pub fn get_chunk(&mut self, chunk_coords: ChunkCoords) -> Option<&Chunk> {
        if !self.loaded_chunks.contains_key(&chunk_coords) {
            let opt = self.get_region(chunk_coords.into());
            let local_x = Self::modulo(chunk_coords.x, 32).abs() as u32;
            let local_z = Self::modulo(chunk_coords.z, 32).abs() as u32;

            if let Some(region) = opt {
                let chunk = region.get_chunk(local_x, local_z, self.global_palette.clone());

                if let Some(chunk) = chunk {
                    self.loaded_chunks.insert(chunk_coords, chunk);
                }
            }
        }
        return self.loaded_chunks.get(&chunk_coords);
    }

    fn get_compressed_chunk(&self, x: i32, z: i32) -> Vec<u8> {
        todo!()
    }

    pub fn get_block(&mut self, world_coords: WorldCoords) -> u32 {
        match self.get_chunk(world_coords.into()) {
            Some(chunk) => {
                let local_block_x: i16 = Self::modulo(world_coords.x, 16) as i16;
                let local_block_z: i16 = Self::modulo(world_coords.z, 16) as i16;
                let block = chunk.get_block(
                    local_block_x.into(),
                    world_coords.y.try_into().unwrap(),
                    local_block_z.into(),
                );
                block
            }
            None => 0,
        }
    }
}
