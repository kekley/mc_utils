use super::chunk::ChunkCoords;
///Coordinates of a single block in "world space"
#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub struct BlockCoords {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl BlockCoords {
    pub fn offset(self, x: i64, y: i64, z: i64) -> BlockCoords {
        BlockCoords {
            x: self.x.saturating_add(x),
            y: self.y.saturating_add(y),
            z: self.z.saturating_add(z),
        }
    }
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
