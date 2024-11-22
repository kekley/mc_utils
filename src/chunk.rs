use std::collections::HashSet;

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

impl ChunkData {
    pub fn from_compound(chunk: NBTCompound) -> Self {
        let data_version = chunk.get_tag("DataVersion").get_int();
        let xpos = chunk.get_tag("xPos").get_int();
        let zpos = chunk.get_tag("zPos").get_int();
        let ypos = chunk.get_tag("yPos").get_int();

        let status = chunk.get_tag("Status").get_string().to_string();
        let last_update = chunk.get_tag("LastUpdate").get_long();

        let sections_tag = chunk.get_tag("sections");

        let list = sections_tag.get_list();

        let sections: Vec<ChunkSection> = list
            .iter()
            .map(|section| {
                let compound = section.get_compound();

                ChunkSection::from_compound(compound)
            })
            .collect();

        Self {
            data_version,
            xpos,
            zpos,
            ypos,
            status: status.to_string(),
            last_update,
            sections: sections.try_into().unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChunkSection {
    ypos: i8,
    block_states: BlockStates,
    block_data: Vec<i64>,
    block_light: Vec<u8>,
    sky_light: Vec<u8>,
}

impl ChunkSection {
    pub fn from_compound(compound: &NBTCompound) -> Self {
        let y = compound.get_tag("Y").get_byte();

        let block_states = compound.get_tag("block_states").get_compound();

        let data = compound.get_tag("data").get_long_array();

        let biomes = compound.get_tag("biomes").get_compound();

        let block_light = compound.get_tag("BlockLight").get_byte_array();

        let sky_light = compound.get_tag("SkyLight").get_byte_array();

        todo!();
    }
}

#[derive(Debug, Clone)]
pub struct BlockStates {
    palette: HashSet<(String, Vec<(String, String)>)>, //(Resource Location, List of Properties)
    data: Vec<i64>,
}

impl BlockStates {
    pub fn from_compound(compound: NBTCompound) -> Self {
        let block_state_list = compound.get_tag("palette").get_list().to_vec();
        let data = compound.get_tag("data").get_long_array().to_vec();

        let vec: Vec<(String, Vec<(String, String)>)> = block_state_list
            .iter()
            .map(|tag| {
                let compound = tag.get_compound();
                let resource = compound.get_tag("Name").get_string().to_string();
                let properties: Vec<(String, String)> = compound
                    .get_tag("Properties")
                    .get_compound()
                    .children
                    .iter()
                    .map(|f| (f.name.clone(), f.tag.get_string().to_string()))
                    .collect();

                (resource, properties)
            })
            .collect();

        let palette = HashSet::from_iter(vec.into_iter());

        BlockStates {
            palette: palette,
            data: data,
        }
    }
}
