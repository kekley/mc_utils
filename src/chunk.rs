use core::str;
use std::{borrow::Borrow, collections::HashSet, io::Cursor};

use bytes::{Buf, Bytes};
use smol_str::SmolStr;

use crate::{nbt_compound::NBTCompound, spider_eye_error::SpiderEyeError, NBTTag};

#[derive(Debug)]
pub struct Chunk {
    data_version: i32,
    pub xpos: i32,
    pub zpos: i32,
    pub ypos: i32,
    status: SmolStr,
    last_update: i64,
    nbt: NBTCompound,
}

impl Chunk {
    pub fn from_slice(mut data: Bytes) -> Result<Self, SpiderEyeError> {
        let compound = NBTCompound::from_borrowed_stream(&mut data)?;
        let binding = compound.get_tag("").unwrap();
        let chunk = binding.get_compound();
        let data_version = chunk.get_tag("DataVersion").unwrap().get_int();
        let xpos = chunk.get_tag("xPos").unwrap().get_int();
        let zpos = chunk.get_tag("zPos").unwrap().get_int();
        let ypos = chunk.get_tag("yPos").unwrap().get_int();

        let status =
            SmolStr::from(str::from_utf8(chunk.get_tag("Status").unwrap().get_string()).unwrap());
        let last_update = chunk.get_tag("LastUpdate").unwrap().get_long();

        Ok(Self {
            nbt: compound,
            data_version,
            xpos,
            zpos,
            ypos,
            status: status,
            last_update,
        })
    }
    pub fn get_data(&self) -> ChunkData<'_> {
        ChunkData::from_compound(&self.nbt)
    }
}

#[derive(Debug)]
pub struct ChunkData<'a> {
    pub sections: [ChunkSection<'a>; 24],
}

impl Default for ChunkData<'_> {
    fn default() -> Self {
        Self {
            sections: Default::default(),
        }
    }
}

impl<'a> ChunkData<'a> {
    pub fn get_block(&self, x: i16, y: i16, z: i16) -> &(&'a Bytes, Vec<(&'a str, &'a Bytes)>) {
        // Calculate the local y coordinate within the section
        let local_y = match y < 0 {
            true => 15 - (y.abs() % 16),
            false => y % 16,
        };
        let section_y = (y as f32 / 16f32).floor();
        // Retrieve the section
        let section = &self
            .sections
            .iter()
            .find(|f| f.ypos == section_y as i8)
            .unwrap();
        // Get the block from the section's block states
        section
            .block_states
            .get_block(x as u16, local_y as u16, z as u16)
    }
    pub fn from_compound(chunk: &'a NBTCompound) -> Self {
        let sections_tag = chunk.get_tag("sections").unwrap();

        let list = sections_tag.get_list();
        let mut sections: Vec<ChunkSection> = vec![];
        list.iter().for_each(|section| {
            let compound = section.get_compound();
            if compound.get_tag("Y").unwrap().get_byte() == -5 {
            } else {
                sections.push(ChunkSection::from_compound(&compound));
            }
        });

        Self {
            sections: sections.try_into().unwrap(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ChunkSection<'a> {
    pub ypos: i8,
    pub block_states: BlockStates<'a>,
    pub block_light: Vec<i8>,
    pub sky_light: Vec<i8>,
}

impl<'a> ChunkSection<'a> {
    pub fn from_compound(compound: &'a NBTCompound) -> Self {
        let y = compound.get_tag("Y").unwrap().get_byte();
        let binding = compound.get_tag("block_states").unwrap();
        let block_states_compound = binding.get_compound();
        let block_light = compound.get_tag("BlockLight");
        let sky_light = compound.get_tag("SkyLight");
        let block_states = BlockStates::from_compound(&block_states_compound);

        let biomes = compound.get_tag("biomes").unwrap().get_compound();

        let block_light = block_light
            .unwrap_or(&NBTTag::ByteArray(vec![]))
            .get_byte_array()
            .to_owned();

        let sky_light = sky_light
            .unwrap_or(&NBTTag::ByteArray(vec![]))
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

#[derive(Debug, Clone, Default)]
pub struct BlockStates<'a> {
    palette: Vec<(&'a Bytes, Vec<(&'a str, &'a Bytes)>)>, //(Resource Location, List of Properties)
    bits_per_block: u8,
    data: Vec<i64>,
}

impl<'a> BlockStates<'a> {
    pub fn from_compound(compound: &'a NBTCompound) -> Self {
        let block_state_list = compound.get_tag("palette").unwrap().get_list();

        let palette: Vec<(&Bytes, Vec<(&str, &Bytes)>)> = block_state_list
            .iter()
            .map(|tag| {
                let compound = tag.get_compound();
                let resource = compound
                    .get_tag("Name")
                    .and_then(|name_tag| Some(name_tag.get_string()))
                    .unwrap();
                let properties: Vec<(&str, &Bytes)> = compound
                    .get_tag("Properties")
                    .and_then(|prop_tag| {
                        prop_tag
                            .get_compound()
                            .children
                            .iter()
                            .map(|f| (f.0.as_str(), f.1.get_string()))
                            .collect::<Vec<_>>()
                            .into()
                    })
                    .unwrap_or_default();

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
            palette,
            bits_per_block,
            data,
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

    pub fn get_block(
        &self,
        x: u16,
        y: u16,
        z: u16,
    ) -> &(&'a bytes::Bytes, Vec<(&'a str, &'a bytes::Bytes)>) {
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
