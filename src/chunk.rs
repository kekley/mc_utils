use core::str;
use std::{
    sync::{Arc, RwLock},
    u32,
};

use bytes::Bytes;
use indexmap::IndexMap;
use smol_str::SmolStr;

use crate::{nbt_compound::NBTCompound, ChunkCoords, NBTTag};

#[derive(Debug)]
pub struct Chunk {
    data_version: i32,
    pub coords: ChunkCoords,
    pub status: SmolStr,
    pub sections: Vec<ChunkSection>,
}

impl Chunk {
    pub fn from_bytes(data: Vec<u8>, palette: Arc<RwLock<IndexMap<String, ()>>>) -> Self {
        let mut bytes = Bytes::from(data);
        let compound = NBTCompound::from_bytes(&mut bytes).expect("Invalid NBT ");
        let binding = compound.get_tag("").expect("Not a Chunk NBT");
        let chunk = binding.get_compound();
        let data_version = chunk
            .get_tag("DataVersion")
            .expect("Not a Chunk NBT")
            .get_int();
        let xpos = chunk.get_tag("xPos").expect("Not a Chunk NBT").get_int();
        let zpos = chunk.get_tag("zPos").expect("Not a Chunk NBT").get_int();

        let status =
            SmolStr::from(str::from_utf8(chunk.get_tag("Status").unwrap().get_string()).unwrap());

        let sections = chunk.get_tag("sections").unwrap().get_list();

        let section_array: Vec<ChunkSection> = sections
            .iter()
            .filter_map(|section| {
                let section_compound = section.get_compound();
                let section = ChunkSection::from_compound(&section_compound, palette.clone());
                if section.ypos >= -4 {
                    Some(section)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        //        section_array.iter().for_each(|f| println!("{}", f.ypos));
        let coords = ChunkCoords::new(xpos.into(), zpos.into());

        Self {
            coords,
            data_version,
            status: status,
            sections: section_array,
        }
    }
    pub fn get_block(&self, x: i16, y: i16, z: i16) -> u32 {
        let local_y = match y < 0 {
            true => 15 - (y.abs() % 16),
            false => y % 16,
        };

        let section_y = (y as f32 / 16f32).floor() as i16;

        self.sections
            .iter()
            .find(|f| f.ypos == section_y as i8)
            .unwrap()
            .get_block(x, local_y, z)
    }
}

impl ChunkSection {
    pub fn get_block(&self, x: i16, y: i16, z: i16) -> u32 {
        let num = *self.data.get((256 * y + 16 * z + x) as usize).unwrap();

        num
    }
}

#[derive(Debug, Clone)]
pub struct ChunkSection {
    pub ypos: i8,
    pub data: [u32; 4096],
}

impl Default for ChunkSection {
    fn default() -> Self {
        Self {
            ypos: 0,
            data: [0u32; 4096],
        }
    }
}

impl ChunkSection {
    pub fn from_compound(
        compound: &NBTCompound,
        palette: Arc<RwLock<IndexMap<String, ()>>>,
    ) -> Self {
        let y = compound.get_tag("Y").unwrap().get_byte();
        if y < -4 || y > 19 {
            return Self {
                ypos: y,
                data: [0u32; 4096],
            };
        }
        let block_states_compound = compound.get_tag("block_states").unwrap().get_compound();
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

        let strings: Vec<&Bytes> = block_states_compound
            .get_tag("palette")
            .unwrap()
            .get_list()
            .iter()
            .map(|f| {
                let block = f.get_compound();

                block.get_tag("Name").unwrap().get_string()
            })
            .collect();

        let read = palette.read().unwrap();
        let mut missing_strs: Vec<String> = Vec::new();
        strings.iter().for_each(|f| {
            if !read.contains_key(str::from_utf8(f).unwrap()) {
                missing_strs.push(str::from_utf8(f).unwrap().to_string());
            }
        });
        drop(read);

        if missing_strs.len() > 0 {
            let mut write = palette.write().unwrap();
            for str in missing_strs {
                write.insert_full(str, ());
            }
            drop(write);
        }

        let data = if strings.len() == 1 {
            &vec![]
        } else {
            block_states_compound
                .get_tag("data")
                .unwrap()
                .get_long_array()
        };

        let bit_size = (f32::log2(strings.len() as f32 - 1.0)).floor() + 1.0;
        //println!("bit size: {}", bit_size);
        let mut temp: [u32; 4096] = std::array::from_fn(|i| {
            let ind = Self::extract_index(&data[..], i as u32, bit_size as u32);
            ind
        });
        let read = palette.read().unwrap();
        temp.iter_mut().for_each(|i| {
            *i = read
                .get_index_of(str::from_utf8(strings[*i as usize]).unwrap())
                .unwrap() as u32;
        });
        drop(read);
        Self {
            ypos: y,
            data: temp,
        }
    }

    #[inline]
    fn extract_index(packed_array: &[i64], index: u32, bit_size: u32) -> u32 {
        if packed_array.len() == 0 {
            return 0;
        }
        let bits_per_index = std::cmp::max(bit_size, 4); // Minimum size of 4 bits
        let indices_per_element = 64 / bits_per_index; // How many indices fit into one 64-bit integer

        // Determine which 64-bit integer contains the desired index
        let element_index = index / indices_per_element;
        let within_element_index = index % indices_per_element;

        // Calculate the bit position within the 64-bit integer
        let bit_position = within_element_index * bits_per_index;

        // Extract the relevant bits
        let mask = (1 << bits_per_index) - 1;
        ((packed_array[element_index as usize] >> bit_position) & mask) as u32
    }

    fn pp(data: &[i64], x: u16, y: u16, z: u16) -> u32 {
        let bits_per_block = 4;
        let idx = Self::extract_index(data, (256 * y + 16 * z + x).into(), bits_per_block);

        data[idx as usize].try_into().unwrap()
    }
}
