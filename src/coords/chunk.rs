use super::block::BlockCoords;

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ChunkCoords {
    pub x: i64,
    pub z: i64,
}

impl ChunkCoords {
    pub fn new(x: i64, z: i64) -> Self {
        Self { x, z }
    }
}

///coordinates from 0-32 local to a region
#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub struct LocalChunkCoords {
    pub x: u8,
    pub z: u8,
}

impl From<ChunkCoords> for LocalChunkCoords {
    fn from(value: ChunkCoords) -> Self {
        LocalChunkCoords {
            x: (value.x % 32) as u8,
            z: (value.z % 32) as u8,
        }
    }
}

///Gets the chunk the block resides in
impl From<BlockCoords> for ChunkCoords {
    fn from(value: BlockCoords) -> Self {
        Self {
            x: value.x >> 4,
            z: value.z >> 4,
        }
    }
}
