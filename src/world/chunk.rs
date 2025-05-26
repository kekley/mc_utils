use std::{fmt::Debug, sync::Arc, u32};

use bumpalo::Bump;
use bytes::Bytes;

use crate::{
    block::Block,
    nbt::{nbt_compound::NBTCompound, nbt_tag::NBTTag},
    palette::BlockPalette,
};

use super::loaded_world::{ChunkCoords, WorldCoords};

#[derive(Clone)]
pub struct Chunk {
    data_version: i32,
    pub coords: ChunkCoords,
    pub sections: SectionTower,
}
impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("data_version", &self.data_version)
            .field("coords", &self.coords)
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
    pub(crate) fn from_nbt_in(nbt_compound: NBTCompound, bump: &Bump) -> Chunk {
        let binding = nbt_compound.get_tag("").expect("Not a Chunk NBT");
        let chunk = binding.get_compound();
        let data_version = chunk
            .get_tag("DataVersion")
            .expect("Not a Chunk NBT")
            .get_int();
        let xpos = chunk.get_tag("xPos").expect("Not a Chunk NBT").get_int();
        let zpos = chunk.get_tag("zPos").expect("Not a Chunk NBT").get_int();

        let sections = chunk.get_tag("sections").unwrap().get_list();

        let section_array: Vec<ChunkSection> = sections
            .iter()
            .filter_map(|section| {
                let section_compound = section.get_compound();
                let section = ChunkSection::from_compound_internal(&section_compound);
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
            sections: sec_tower,
        }
    }
    pub fn get_local_block(&self, x: usize, y: isize, z: usize) -> Option<&Block> {
        let sections = &self.sections;
        if y > self.sections.y_max() || y < self.sections.y_min() {
            return None;
        }
        let sec = sections.get_section_for_y(y as isize)?;
        let sec_y = (y - sec.ypos as isize * 16) as usize;
        sec.get_block(x, sec_y, z)
    }
    pub fn get_world_block(&self, world_coords: WorldCoords) -> Option<&Block> {
        // Assuming world_coords.x and world_coords.z are integer types (e.g., i64, i32).
        // The operation `& 15` computes `value % 16` correctly for both positive and negative values,
        // resulting in a value in the range [0, 15].
        // This is much faster than a division-based modulo.

        // If world_coords.x is i64, (world_coords.x & 15) is an i64 in [0, 15].
        // Casting this to i16 is safe and preserves the value.
        let local_block_x: i16 = (world_coords.x & 15) as i16;
        let local_block_z: i16 = (world_coords.z & 15) as i16;

        // The .try_into().unwrap() calls:
        // For local_block_x and local_block_z (which are 0..15):
        // If the target type for get_local_block (e.g., usize) can hold 0..15,
        // this conversion is safe and typically well-optimized.
        let final_local_x = match local_block_x.try_into() {
            Ok(val) => val,
            Err(_) => {
                // This path should ideally not be hit if the target type is usize or similar.
                // If it can, panicking via unwrap() is costly. Consider returning None.
                // For now, let's assume it matches the original unwrap() behavior if types are compatible.
                unreachable!("local_block_x (0-15) should always convert to target type");
            }
        };

        // For world_coords.y:
        // This conversion's safety and performance depend on the type of world_coords.y
        // and the type expected by get_local_block. If world_coords.y can be out of range
        // for the target type, .unwrap() will panic, which is slow.
        // Consider returning None earlier if y can be invalid.
        let final_local_y = match world_coords.y.try_into() {
            Ok(val) => val,
            Err(_) => {
                // If invalid y values are possible and not exceptional, handle them gracefully.
                // For example, return None instead of panicking:
                // return None;
                // For this optimization, we assume current .unwrap() behavior is intended for valid inputs.
                panic!("world_coords.y out of range for target type"); // or keep .unwrap()
            }
        };

        let final_local_z = match local_block_z.try_into() {
            Ok(val) => val,
            Err(_) => {
                unreachable!("local_block_z (0-15) should always convert to target type");
            }
        };

        // If you are certain the try_into() calls will not fail (i.e., the values always fit),
        // the original .unwrap() is fine. For local_block_x and local_block_z, if the target
        // type in get_local_block is usize, you could even do `as usize` directly:
        // let final_local_x = (world_coords.x & 15) as usize;
        // let final_local_z = (world_coords.z & 15) as usize;
        // let final_local_y = world_coords.y.try_into().unwrap(); // Keep as is or adapt based on Y's type & constraints

        self.get_local_block(final_local_x, final_local_y, final_local_z)
    }

    // Make sure your WorldCoords struct field types are appropriate.
    // For example:
    // pub struct WorldCoords {
    //     pub x: i64, // or i32, etc.
    //     pub y: i64, // or i32, u16, etc. This type is important for try_into()
    //     pub z: i64, // or i32, etc.
    // }
}

#[derive(Debug, Clone)]
pub struct ChunkSection {
    pub ypos: i8,
    pub(crate) block_palette: BlockPalette,
    pub block_data: [u32; 4096],
    pub biome_data: [u32; 4096],
}

impl ChunkSection {
    pub(crate) fn with_palette_from_compound(interner: &Arc<ThreadedRodeo>) -> Self {
        unimplemented!()
    }
    pub(crate) fn from_compound_internal(compound: &NBTCompound) -> Self {
        let mut palette = BlockPalette::new_inner(interner);
        let y = compound.get_tag("Y").unwrap().get_byte();
        //ignore non-vanilla world heights for now
        //FIXME
        if y < -4 || y > 19 {
            return Self {
                block_palette: BlockPalette::new_inner(interner),
                ypos: y,
                block_data: [0u32; 4096],
                biome_data: [0u32; 4096],
            };
        }
        let block_states_compound = compound.get_tag("block_states").unwrap().get_compound();
        let block_light = compound.get_tag("BlockLight");
        let sky_light = compound.get_tag("SkyLight");

        let biomes_tag = compound.get_tag("biomes").unwrap().get_compound();
        let biome_palette = biomes_tag.get_tag("palette").unwrap().get_list();
        biome_palette.iter().for_each(|entry| {
            let biome_resource = entry.get_string();
            //dbg!(biome_resource);
        });

        let block_light = block_light
            .unwrap_or(&NBTTag::ByteArray(Bytes::new()))
            .get_byte_array()
            .to_owned();

        let sky_light = sky_light
            .unwrap_or(&NBTTag::ByteArray(Bytes::new()))
            .get_byte_array()
            .to_owned();

        let block_states: Vec<Block> = block_states_compound
            .get_tag("palette")
            .unwrap()
            .get_list()
            .iter()
            .map(|f| {
                let block = f.get_compound();
                let block_name = block.get_tag("Name").unwrap().get_string();
                let block_name_spur = interner.get_or_intern(block_name);
                let mut props = Vec::with_capacity(2);
                let block_states: InternedBlockState = block
                    .get_tag("Properties")
                    .map(|properties| {
                        properties.get_compound().children.iter().for_each(|f| {
                            let state_name = f.0.clone();
                            let state_value = interner.get_or_intern(f.1.get_string());
                            props.push((state_name, state_value));
                        });
                        InternedBlockState { properties: props }
                    })
                    .unwrap_or(BlockState { properties: vec![] });

                Block {
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

        palette.block_states = block_states;

        let block_data: [u32; 4096] = std::array::from_fn(|i| {
            let ind = Self::extract_index(&data[..], i as u32, bit_size as u32);
            ind
        });

        let biome_data = [0u32; 4096];

        Self {
            block_palette: palette,
            ypos: y,
            block_data,
            biome_data,
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
    #[inline(always)]
    pub fn get_block(&self, x: usize, sec_y: usize, z: usize) -> Option<&Block> {
        let num = self
            .block_data
            .get((sec_y * 16 * 16 + z * 16 + x) as usize)
            .expect("invalid index");
        self.block_palette.get(*num)
    }
    fn pp(data: &[i64], x: u16, y: u16, z: u16) -> u32 {
        let bits_per_block = 4;
        let idx = Self::extract_index(data, (256 * y + 16 * z + x).into(), bits_per_block);

        data[idx as usize].try_into().unwrap()
    }
}
