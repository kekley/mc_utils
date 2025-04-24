use std::{fmt::Debug, sync::Arc, u32};

use bytes::Bytes;
use lasso::ThreadedRodeo;
use smol_str::SmolStr;

use crate::{
    block::{self, InternedBlock},
    block_states::InternedBlockState,
    nbt::{nbt_compound::NBTCompound, nbt_tag::NBTTag},
    palette::BlockPalette,
};

use super::loaded_world::{modulo, ChunkCoords, WorldCoords};

#[derive(Clone)]
pub struct Chunk {
    data_version: i32,
    pub coords: ChunkCoords,
    pub status: SmolStr,
    pub sections: SectionTower,
}
impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("data_version", &self.data_version)
            .field("coords", &self.coords)
            .field("status", &self.status)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct SectionTower {
    sections: Vec<ChunkSection>,
    map: Vec<Option<usize>>,
    y_min: isize,
    y_max: isize,
}
const fn y_to_index(y: isize, y_min: isize) -> u8 {
    ((y - y_min) >> 4) as u8
}

impl SectionTower {
    pub fn get_section_for_y(&self, y: isize) -> Option<&ChunkSection> {
        if y >= self.y_max || y < self.y_min {
            return None;
        }

        let lookup_index = y_to_index(y, self.y_min);

        let section_index = *self.map.get(lookup_index as usize)?;
        self.sections.get(section_index?)
    }
    pub fn y_min(&self) -> isize {
        self.y_min
    }

    pub fn y_max(&self) -> isize {
        self.y_max
    }
}
impl Chunk {
    pub(crate) fn from_nbt_internal(
        nbt_compound: NBTCompound,
        interner: &Arc<ThreadedRodeo>,
    ) -> Chunk {
        let binding = nbt_compound.get_tag("").expect("Not a Chunk NBT");
        let chunk = binding.get_compound();
        let data_version = chunk
            .get_tag("DataVersion")
            .expect("Not a Chunk NBT")
            .get_int();
        let xpos = chunk.get_tag("xPos").expect("Not a Chunk NBT").get_int();
        let zpos = chunk.get_tag("zPos").expect("Not a Chunk NBT").get_int();

        let status = SmolStr::from(chunk.get_tag("Status").unwrap().get_string());

        let sections = chunk.get_tag("sections").unwrap().get_list();

        let section_array: Vec<ChunkSection> = sections
            .iter()
            .filter_map(|section| {
                let section_compound = section.get_compound();
                let section = ChunkSection::from_compound_internal(&section_compound, interner);
                if section.ypos >= -4 {
                    Some(section)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let lowest_section = section_array
            .iter()
            .min_by_key(|s| s.ypos)
            .expect("empty section array");

        let min = lowest_section.ypos as isize;
        let max = section_array
            .iter()
            .max_by_key(|s| s.ypos)
            .map(|s| s.ypos)
            .unwrap() as isize;
        let mut sparse_sections = vec![None; (1 + max - min) as usize];

        for (i, sec) in section_array.iter().enumerate() {
            let sec_index = (sec.ypos as isize - min) as usize;

            sparse_sections[sec_index] = Some(i);
        }

        let sec_tower = SectionTower {
            sections: section_array,
            map: sparse_sections,
            y_min: 16 * min,
            y_max: 16 * (max + 1),
        };

        let coords = ChunkCoords::new(xpos.into(), zpos.into());

        Self {
            coords,
            data_version,
            status: status,
            sections: sec_tower,
        }
    }

    pub fn get_local_block(&self, x: usize, y: isize, z: usize) -> Option<&InternedBlock> {
        let sections = &self.sections;
        if y > self.sections.y_max() || y < self.sections.y_min() {
            return None;
        }
        let sec = sections.get_section_for_y(y as isize)?;
        let sec_y = (y - sec.ypos as isize * 16) as usize;
        sec.get_block(x, sec_y, z)
    }
    pub fn get_world_block(&self, world_coords: WorldCoords) -> Option<&InternedBlock> {
        let local_block_x: i16 = modulo(world_coords.x, 16) as i16;
        let local_block_z: i16 = modulo(world_coords.z, 16) as i16;
        let block = self.get_local_block(
            local_block_x.try_into().unwrap(),
            world_coords.y.try_into().unwrap(),
            local_block_z.try_into().unwrap(),
        );

        block
    }
}

impl ChunkSection {
    pub fn get_block(&self, x: usize, sec_y: usize, z: usize) -> Option<&InternedBlock> {
        let num = self
            .data
            .get((sec_y * 16 * 16 + z * 16 + x) as usize)
            .expect("invalid index");
        self.palette.get(*num)
    }
}

#[derive(Debug, Clone)]
pub struct ChunkSection {
    pub ypos: i8,
    pub(crate) palette: BlockPalette,
    pub data: [u32; 4096],
}

impl ChunkSection {
    pub(crate) fn from_compound_internal(
        compound: &NBTCompound,
        interner: &Arc<ThreadedRodeo>,
    ) -> Self {
        let mut palette = BlockPalette::new_inner(interner);
        let y = compound.get_tag("Y").unwrap().get_byte();
        //ignore non-vanilla world heights for now
        //FIXME
        if y < -4 || y > 19 {
            return Self {
                palette: BlockPalette::new_inner(interner),
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

        let block_states: Vec<InternedBlock> = block_states_compound
            .get_tag("palette")
            .unwrap()
            .get_list()
            .iter()
            .map(|f| {
                let block = f.get_compound();
                let block_name = block.get_tag("Name").unwrap().get_string();
                let block_name_spur = interner.get_or_intern(block_name);
                let mut props = vec![];
                let block_states: InternedBlockState = block
                    .get_tag("properties")
                    .map(|properties| {
                        properties.get_compound().children.iter().for_each(|f| {
                            let state_name = f.0.clone();
                            let state_value = interner.get_or_intern(f.1.get_string());
                            props.push((state_name, state_value));
                        });
                        InternedBlockState { properties: props }
                    })
                    .unwrap_or(InternedBlockState { properties: vec![] });

                InternedBlock {
                    block_name: block_name_spur,
                    properties: block_states,
                }
            })
            .collect();
        let data = if block_states.len() == 1 {
            &vec![]
        } else {
            &block_states_compound
                .get_tag("data")
                .unwrap()
                .get_long_array()
        };

        let bit_size = (f32::log2(block_states.len() as f32 - 1.0)).floor() + 1.0;

        let temp: [u32; 4096] = std::array::from_fn(|i| {
            let ind = Self::extract_index(&data[..], i as u32, bit_size as u32);
            ind
        });

        block_states.into_iter().for_each(|state| {
            /*             let resolved_block = state.resolve(interner);
            if resolved_block.block_name != "minecraft:air" {
                dbg!(resolved_block);
            } */

            palette.insert_block(state);
        });

        Self {
            palette,
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
