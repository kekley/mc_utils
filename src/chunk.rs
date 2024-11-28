use std::collections::HashSet;

use bytes::Buf;
use smol_str::SmolStr;

use crate::{nbt_compound::NBTCompound, spider_eye_error::SpiderEyeError, NBTTag};

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
    status: SmolStr,
    last_update: i64,
    pub sections: [ChunkSection; 25],
}

impl ChunkData {
    pub fn from_compound(chunk: NBTCompound) -> Self {
        let binding = chunk.get_tag("").unwrap();
        let chunk = binding.get_compound();
        let data_version = chunk.get_tag("DataVersion").unwrap().get_int();
        let xpos = chunk.get_tag("xPos").unwrap().get_int();
        let zpos = chunk.get_tag("zPos").unwrap().get_int();
        let ypos = chunk.get_tag("yPos").unwrap().get_int();

        let status = chunk.get_tag("Status").unwrap().get_string().clone();
        let last_update = chunk.get_tag("LastUpdate").unwrap().get_long();

        let sections_tag = chunk.get_tag("sections").unwrap();

        let list = sections_tag.get_list();
        print!("{:?}", list);
        let sections: Vec<ChunkSection> = list
            .iter()
            .map(|section| {
                let compound = section.get_compound();
                if compound.get_tag("Y").unwrap().get_byte() == -5 {
                    return ChunkSection {
                        ypos: -5,
                        block_states: BlockStates {
                            palette: vec![(SmolStr::new("minecraft:air"), vec![])],
                            bits_per_block: 4,
                            data: vec![],
                        },
                        block_light: vec![],
                        sky_light: vec![],
                    };
                }
                ChunkSection::from_compound(compound)
            })
            .collect();

        Self {
            data_version,
            xpos,
            zpos,
            ypos,
            status: status,
            last_update,
            sections: sections.try_into().unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChunkSection {
    pub ypos: i8,
    pub block_states: BlockStates,
    pub block_light: Vec<i8>,
    pub sky_light: Vec<i8>,
}

impl ChunkSection {
    pub fn from_compound(compound: &NBTCompound) -> Self {
        let y = compound.get_tag("Y").unwrap().get_byte();
        println!("{:?}", compound);
        let binding = compound.get_tag("block_states").unwrap();
        let block_states_compound = binding.get_compound();
        let block_states = BlockStates::from_compound(block_states_compound.clone());

        let biomes = compound.get_tag("biomes").unwrap().get_compound();

        let block_light = compound.get_tag("BlockLight");

        let sky_light = compound.get_tag("SkyLight");

        let block_light = block_light
            .unwrap_or(NBTTag::ByteArray(vec![]))
            .get_byte_array()
            .to_owned();

        let sky_light = sky_light
            .unwrap_or(NBTTag::ByteArray(vec![]))
            .get_byte_array()
            .to_owned();

        ChunkSection {
            ypos: y,
            block_states: block_states,
            block_light: block_light,
            sky_light: sky_light,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockStates {
    palette: Vec<(SmolStr, Vec<(SmolStr, SmolStr)>)>, //(Resource Location, List of Properties)
    bits_per_block: u8,
    data: Vec<i64>,
}

impl BlockStates {
    pub fn from_compound(compound: NBTCompound) -> Self {
        let block_state_list = compound.get_tag("palette").unwrap().get_list().to_vec();

        let palette: Vec<(SmolStr, Vec<(SmolStr, SmolStr)>)> = block_state_list
            .iter()
            .map(|tag| {
                let compound = tag.get_compound();
                let resource = compound.get_tag("Name").unwrap().get_string().clone();
                let properties: Vec<(SmolStr, SmolStr)> = compound
                    .get_tag("Properties")
                    .or_else(|| Some(NBTTag::Compound(NBTCompound { children: vec![] })))
                    .unwrap()
                    .get_compound()
                    .children
                    .iter()
                    .map(|f| (f.name.clone(), f.tag.get_string().clone()))
                    .collect();

                (resource, properties)
            })
            .collect();

        let data = if palette.len() > 1 {
            compound.get_tag("data").unwrap().get_long_array().to_vec()
        } else {
            vec![]
        };
        let bits_per_block = if ((palette.len() as f64 + 1.0).log2().ceil() as u8) <= 4 {
            4
        } else {
            (palette.len() as f64 + 1.0).log2().ceil() as u8
        };

        BlockStates {
            palette: palette,
            bits_per_block: bits_per_block,
            data: data,
        }
    }
    #[inline]
    fn extract_index(packed_array: &[i64], index: usize, bit_size: usize) -> usize {
        let bits_per_index = std::cmp::max(bit_size, 4); // Minimum size of 4 bits
        let indices_per_element = 64 / bits_per_index; // How many indices fit into one 64-bit integer

        // Determine which 64-bit integer contains the desired index
        let element_index = index / indices_per_element;
        let within_element_index = index % indices_per_element;

        // Calculate the bit position within the 64-bit integer
        let bit_position = within_element_index * bits_per_index;

        // Extract the relevant bits
        let mask = (1 << bits_per_index) - 1;
        ((packed_array[element_index] >> bit_position) & mask) as usize
    }

    pub fn get_block(&self, x: u16, y: u16, z: u16) -> &(SmolStr, Vec<(SmolStr, SmolStr)>) {
        if self.palette.len() == 1 {
            unsafe { &self.palette.get_unchecked(0) }
        } else {
            let idx = Self::extract_index(
                self.data.as_slice(),
                (256 * y + 16 * z + x) as usize,
                self.bits_per_block as usize,
            );

            &self.palette[idx]
        }
    }
}
