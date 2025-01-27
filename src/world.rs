use std::{
    fs,
    hash::{BuildHasher, Hash},
    sync::{Arc, RwLock},
};

use dashmap::DashMap;
use fxhash::{FxBuildHasher, FxHasher};
use hashbrown::HashMap;
use smol_str::SmolStr;

use crate::{block_states::BlockState, palette::Palette, Chunk, Region};
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
    pub regions: HashMap<RegionCoords, Region, FxBuildHasher>,
    pub cached_chunks: DashMap<ChunkCoords, Option<Arc<Chunk>>, FxBuildHasher>,
    pub global_palette: Palette<BlockState>,
}

impl World {
    pub fn new(folder_path: &str) -> Self {
        let mut palette = Palette::new();
        let air = BlockState {
            block: "minecraft:air".into(),
            properties: None,
        };
        palette.insert(air);
        let mut temp = Self {
            regions: HashMap::with_hasher(FxBuildHasher::default()),
            cached_chunks: DashMap::with_hasher(FxBuildHasher::default()),
            global_palette: palette,
        };
        let read_dir = fs::read_dir(folder_path).expect("could not find folder");
        for dir in read_dir {
            let entry = dir.unwrap();

            let path = entry.path();
            if path.is_file() && path.extension().unwrap() == "mca" {
                if let Ok(region) =
                    Region::from_file(path.to_str().unwrap().to_owned(), &temp.global_palette)
                {
                    temp.regions.insert(region.coords, region);
                }
            }
        }
        temp
    }

    pub fn get_region(&self, region_coords: RegionCoords) -> Option<&Region> {
        self.regions.get(&region_coords)
    }
    pub fn modulo(a: i64, b: i64) -> i64 {
        let r = a % b;
        if r < 0 {
            r + b
        } else {
            r
        }
    }

    pub fn get_chunk_cached(&self, chunk_coords: ChunkCoords) -> Option<Arc<Chunk>> {
        let res = self.cached_chunks.get(&chunk_coords);
        if res.is_some() {
            let chunk = res.unwrap();
            return chunk.clone();
        }

        let opt = self.get_region(chunk_coords.into());
        let local_x = Self::modulo(chunk_coords.x, 32).abs() as u32;
        let local_z = Self::modulo(chunk_coords.z, 32).abs() as u32;
        if let Some(region) = opt {
            let chunk = region.get_chunk(local_x, local_z, &self.global_palette);
            if chunk.is_none() {
                self.cached_chunks.insert(chunk_coords, None);
                return None;
            } else {
                let arc = Arc::new(chunk.unwrap());
                self.cached_chunks.insert(chunk_coords, Some(arc.clone()));
                return Some(arc);
            }
        }

        None
    }

    pub fn get_block(&self, world_coords: WorldCoords) -> Option<u32> {
        match self.get_chunk_cached(world_coords.into()) {
            Some(chunk) => {
                let local_block_x: i16 = Self::modulo(world_coords.x, 16) as i16;
                let local_block_z: i16 = Self::modulo(world_coords.z, 16) as i16;
                let block = chunk.get_local_block(
                    local_block_x.try_into().unwrap(),
                    world_coords.y.try_into().unwrap(),
                    local_block_z.try_into().unwrap(),
                );
                block
            }
            None => None,
        }
    }
}
