use super::{block::BlockCoords, chunk::ChunkCoords};

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub struct RegionCoords {
    pub x: i64,
    pub z: i64,
}

///Get the region the chunk resides in
impl From<ChunkCoords> for RegionCoords {
    #[inline]
    fn from(value: ChunkCoords) -> Self {
        Self {
            x: value.x >> 5,
            z: value.z >> 5,
        }
    }
}

///Get the region the block resides in
impl From<BlockCoords> for RegionCoords {
    #[inline]
    fn from(value: BlockCoords) -> Self {
        Self {
            x: value.x >> 10,
            z: value.z >> 10,
        }
    }
}
