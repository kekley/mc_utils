use std::{hash::Hash, sync::Arc};

use fxhash::FxBuildHasher;
use log::info;

use crate::palette::BlockPalette;

use super::{
    chunk::Chunk,
    region::{LazyRegion, LoadedRegion},
    world_error::WorldError,
};
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
            x: value.x >> 10,
            z: value.z >> 10,
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
    region_folder: String,
}

impl World {
    pub(crate) fn new(folder_path: &str) -> Result<World, WorldError> {
        info!("Opening world folder");
        let temp = Self {
            region_folder: folder_path.to_owned(),
        };
        dbg!(folder_path);

        Ok(temp)
    }

    pub fn get_region_lazy(&self, region_coords: RegionCoords) -> Option<LazyRegion> {
        info!("Creating lazy region loader for region {region_coords:?}");
        let x = region_coords.x;
        let z = region_coords.z;
        let mut path_str = String::new();
        path_str.push_str(&self.region_folder);
        path_str.push_str(&format!("/r.{x}.{z}.mca"));
        let file_path = path_str;
        LazyRegion::new(&file_path, &self.interner).ok()
    }

    pub fn load_region(&self, region_coords: RegionCoords) -> Option<LoadedRegion> {
        info!("Creating loaded region for: {region_coords:?}");
        self.get_region_lazy(region_coords).map(|f| (&f).into())
    }
}
