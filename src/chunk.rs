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
    pub fn from_bytes(mut data: Bytes) -> Result<Self, SpiderEyeError> {
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
    pub fn get_data(&self) -> ChunkData {
        ChunkData::from_compound(&self.nbt)
    }
}

#[derive(Debug)]
pub struct ChunkData {
    pub sections: [ChunkSection; 24],
}

impl Default for ChunkData {
    fn default() -> Self {
        Self {
            sections: Default::default(),
        }
    }
}

impl ChunkData {
    pub fn get_block(&self, x: i16, y: i16, z: i16) -> u32 {
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
        section.data.get_block(x as u16, local_y as u16, z as u16)
    }
    pub fn from_compound(chunk: &NBTCompound) -> Self {
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

#[derive(Debug, Clone)]
pub struct ChunkSection {
    pub ypos: i8,
    pub data: [u32; 4096],
    pub block_light: [i8; 4096],
    pub sky_light: [i8; 4096],
}

impl Default for ChunkSection {
    fn default() -> Self {
        Self {
            ypos: 0,
            data: [0u32; 4096],
            block_light: [0i8; 4096],
            sky_light: [0i8; 4096],
        }
    }
}

impl ChunkSection {
    pub fn from_compound(compound: &NBTCompound) -> Self {
        let y = compound.get_tag("Y").unwrap().get_byte();
        let binding = compound.get_tag("block_states").unwrap();
        let block_states_compound = binding.get_compound();
        let block_light = compound.get_tag("BlockLight");
        let sky_light = compound.get_tag("SkyLight");

        let biomes = compound.get_tag("biomes").unwrap().get_compound();

        let block_light = block_light
            .unwrap_or(&NBTTag::ByteArray(Bytes::new()))
            .get_byte_array()
            .to_owned();

        let sky_light = sky_light
            .unwrap_or(&NBTTag::ByteArray(Bytes::new()))
            .get_byte_array()
            .to_owned();

        ChunkSection {
            ypos: y,
            block_states: block_states,
            block_light: block_light,
            sky_light: sky_light,
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

    pub fn get_block(&self, x: u16, y: u16, z: u16) -> u32 {
        let idx = Self::extract_index(
            self.data.as_slice(),
            (256 * y + 16 * z + x) as usize,
            self.bits_per_block as usize,
        );

        self.data[idx];
    }
}
