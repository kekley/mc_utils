use bytes::Buf;

use crate::{nbt_compound::NBTCompound, spider_eye_error::SpiderEyeError};

#[derive(Debug, Clone)]
pub struct Chunk {
    pub data: NBTCompound,
}

impl Chunk {
    pub fn from_data(data: &mut dyn Buf) -> Result<Self, SpiderEyeError> {
        Ok(Self {
            data: NBTCompound::from_borrowed_stream(data)?,
        })
    }
}

pub struct ChunkData {
    data_version: i32,
    xpos: i32,
    zpos: i32,
    ypos: i32,
    status: String,
    last_update: i64,
    sections: [ChunkSection; 25],
}

pub struct ChunkSection {
    ypos: i8,
    block_palette: BlockPalette,
    block_data: Vec<i64>,
}

pub struct BlockPalette {
    block_states: Vec<BlockState>,
}

pub struct BlockState {
    block_name: String,
    properties: Vec<(String, String)>,
}

pub struct BiomePalette {}
