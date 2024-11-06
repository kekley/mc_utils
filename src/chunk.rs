use std::io::{Cursor, Read};

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
}
