use super::block::BlockCoords;
///The coordinates of a chunk in chunk units
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

///The coordinates of a chunk from the frame of reference of the region it resides in. In the
///range 0-31
#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub struct RegionLocalChunkCoords {
    pub x: u8,
    pub z: u8,
}

impl From<ChunkCoords> for RegionLocalChunkCoords {
    fn from(value: ChunkCoords) -> Self {
        RegionLocalChunkCoords {
            x: (value.x.abs() % 32) as u8,
            z: (value.z.abs() % 32) as u8,
        }
    }
}

///Gets the chunk coordinates for the chunk the block resides in
impl From<BlockCoords> for ChunkCoords {
    fn from(value: BlockCoords) -> Self {
        Self {
            x: value.x >> 4,
            z: value.z >> 4,
        }
    }
}
