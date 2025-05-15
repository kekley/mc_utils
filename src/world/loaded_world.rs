use std::{fs, hash::Hash, sync::Arc};

use anyhow::Context;
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use lasso::{Spur, ThreadedRodeo};

use crate::{
    block::InternedBlock,
    palette::{BlockPalette, InternerType},
    MCResourceLoader,
};

use super::{
    chunk::Chunk,
    region::{LazyRegion, LoadedRegion},
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
    chunk_cache: DashMap<ChunkCoords, Option<Chunk>, FxBuildHasher>,
    pub global_palette: BlockPalette,
}

impl World {
    pub(crate) fn new(folder_path: &str, interner: &Arc<ThreadedRodeo>) -> anyhow::Result<World> {
        let palette = BlockPalette::new_inner(interner);
        let mut temp = Self {
            interner: InternerType::External(interner.clone()),
            regions: HashMap::with_hasher(FxBuildHasher::default()),
            global_palette: palette,
            chunk_cache: DashMap::with_hasher(FxBuildHasher::default()),
        };

        let read_dir = fs::read_dir(folder_path)
            .context(format!("Folder path {folder_path} did not exist"))?;
        for dir in read_dir {
            if let Ok(entry) = dir {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|extension| extension == "mca") {
                    let region = LazyRegion::new(
                        path.to_str()
                            .context(format!("Path: {path:?} could not be converted to string"))?,
                        interner,
                    );
                    if let Ok(region) = region {
                        temp.regions.insert(region.coords, region);
                    }
                }
            }
        }
        Ok(temp)
    }

    pub fn get_region_lazy(&self, region_coords: RegionCoords) -> Option<&LazyRegion> {
        self.regions.get(&region_coords)
    }

    pub fn load_region(&self, region_coords: RegionCoords) -> Option<LoadedRegion> {
        self.get_region_lazy(region_coords).map(|f| f.into())
    }

    pub fn get_block(&self, block_coords: &WorldCoords) -> Option<InternedBlock> {
        let chunk_coords = ChunkCoords::from(*block_coords);
        if let Some(chunk) = self.chunk_cache.get(&chunk_coords) {
            return chunk.as_ref()?.get_world_block(*block_coords).cloned();
        } else {
            let region_coords = RegionCoords::from(*block_coords);
            let region = self.regions.get(&region_coords)?;
            let loaded = LoadedRegion::from(region);
            let mut chunks = loaded.get_all_chunks();
            for z in 0..32 {
                for x in 0..32 {
                    let coords =
                        ChunkCoords::new((region_coords.x * 32) + x, (region_coords.z * 32) + z);
                    let chunk = chunks[(x + 32 * z) as usize].take();
                    self.chunk_cache.insert(coords, chunk);
                }
            }
            return self.get_block(block_coords);
        }
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

#[test]

pub fn world_loading() {
    let loader = MCResourceLoader::new();
    let interner = &loader.rodeo;
    let world = loader.open_world("./test_world").unwrap();
    for z in 0..32 {
        for x in 0..32 {
            let block = world.get_block(&WorldCoords { x: x, y: -63, z: z });
            if let Some(block) = block {
                let InternedBlock {
                    block_name,
                    properties,
                } = block;
            }
        }
    }
}
