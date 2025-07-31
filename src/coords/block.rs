use super::chunk::ChunkCoords;

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub struct BlockCoords {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

///Get the block closest to 0,0 in the chunk
impl From<ChunkCoords> for BlockCoords {
    fn from(value: ChunkCoords) -> Self {
        Self {
            x: 16 * value.x,
            y: 0,
            z: 16 * value.z,
        }
    }
}
