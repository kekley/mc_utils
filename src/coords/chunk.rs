use super::block::BlockCoords;

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ChunkCoords {
    pub x: i64,
    pub z: i64,
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub struct LocalChunkCoords {
    pub x: u8,
    pub z: u8,
}

impl From<ChunkCoords> for LocalChunkCoords {
    fn from(value: ChunkCoords) -> Self {
        let abs_x = value.x.abs();

        let abs_z = value.z.abs();

        let x = (abs_x % 16) as u8;

        let z = (abs_z % 16) as u8;

        LocalChunkCoords { x, z }
    }
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
