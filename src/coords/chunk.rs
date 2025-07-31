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

///Gets the chunk the block resides in
impl From<BlockCoords> for ChunkCoords {
    fn from(value: BlockCoords) -> Self {
        Self {
            x: value.x >> 4,
            z: value.z >> 4,
        }
    }
}
