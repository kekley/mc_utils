use std::{fs, hash::Hash, sync::Arc};

use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use lasso::{Spur, ThreadedRodeo};

use crate::palette::{BlockPalette, InternerType};

use super::region::{LazyRegion, LoadedRegion};
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

pub type InternedBlockName = Spur;
#[derive(Debug)]
pub struct World {
    interner: InternerType,
    pub regions: HashMap<RegionCoords, LazyRegion, FxBuildHasher>,
    pub global_palette: BlockPalette,
}

impl World {
    pub(crate) fn new(folder_path: &str, interner: &Arc<ThreadedRodeo>) -> Self {
        let palette = BlockPalette::new_inner(interner);
        let mut temp = Self {
            interner: InternerType::External(interner.clone()),
            regions: HashMap::with_hasher(FxBuildHasher::default()),
            global_palette: palette,
        };

        let read_dir = fs::read_dir(folder_path).expect("could not find folder");
        for dir in read_dir {
            let entry = dir.unwrap();

            let path = entry.path();
            if path.is_file() && path.extension().unwrap() == "mca" {
                if let Ok(region) = LazyRegion::new(path.to_str().unwrap(), interner) {
                    temp.regions.insert(region.coords, region);
                }
            }
        }
        temp
    }

    pub fn get_region_lazy(&self, region_coords: RegionCoords) -> Option<&LazyRegion> {
        self.regions.get(&region_coords)
    }

    pub fn load_region(&self, region_coords: RegionCoords) -> Option<LoadedRegion> {
        self.get_region_lazy(region_coords).map(|f| f.into())
    }
}
pub fn modulo(a: i64, b: i64) -> i64 {
    let r = a % b;
    if r < 0 {
        r + b
    } else {
        r
    }
}
