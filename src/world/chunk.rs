use std::fmt::Debug;

use super::{
    chunk_error::ChunkError,
    loaded_world::{ChunkCoords, WorldCoords},
};
use crate::{
    block::{Block, BlockName},
    owned::nbt_compound::{NBTCompound, NBTTag},
};

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

pub trait ExpectTag {
    fn expect_tag(&self, error_text: &str) -> Result<&NBTTag, ChunkError>;
}

impl ExpectTag for Option<&NBTTag> {
    fn expect_tag(&self, error_text: &str) -> Result<&NBTTag, ChunkError> {
        self.ok_or(ChunkError {
            kind: super::chunk_error::ChunkErrorKind::InvalidNBT(format!("{error_text}")),
        })
    }
}

impl Chunk {
    pub(crate) fn from_nbt_in(nbt_compound: NBTCompound) -> Result<Chunk, ChunkError> {
        let binding = nbt_compound.get_tag("");
        let tag = binding.expect_tag("Chunks must start with an empty name compound tag")?;
        let chunk = tag.get_compound();
        let data_version = chunk
            .get_tag("DataVersion")
            .expect("Not a Chunk NBT")
            .get_int();
        let xpos = chunk.get_tag("xPos").expect("Not a Chunk NBT").get_int();
        let zpos = chunk.get_tag("zPos").expect("Not a Chunk NBT").get_int();

        let sections = chunk.get_tag("sections").unwrap().get_list();

        let section_array: Vec<ChunkSection> = sections
            .into_iter()
            .filter_map(|section| {
                let section_compound = section.get_compound().clone();
                let section = ChunkSection::from_compound_internal(section_compound).unwrap();
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

        Ok(Self {
            coords,
            data_version,
            sections: sec_tower,
        })
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
        let local_block_x: i16 = (world_coords.x & 15) as i16;
        let local_block_z: i16 = (world_coords.z & 15) as i16;

        let final_local_x = match local_block_x.try_into() {
            Ok(val) => val,
            Err(_) => {
                unreachable!("local_block_x (0-15) should always convert to target type");
            }
        };
        let final_local_y = match world_coords.y.try_into() {
            Ok(val) => val,
            Err(_) => {
                panic!("world_coords.y out of range for target type");
            }
        };

        let final_local_z = match local_block_z.try_into() {
            Ok(val) => val,
            Err(_) => {
                unreachable!("local_block_z (0-15) should always convert to target type");
            }
        };
        self.get_local_block(final_local_x, final_local_y, final_local_z)
    }
}

#[derive(Debug, Clone)]
pub struct ChunkSection {
    pub ypos: i8,
    pub block_data: [u32; 4096],
    pub biome_data: [u32; 4096],
}

impl ChunkSection {
    pub(crate) fn from_compound_internal(compound: NBTCompound) -> Result<Self, ChunkError> {
        let y = compound.get_tag("Y").unwrap().get_byte();
        //ignore non-vanilla world heights for now
        //FIXME
        if !(-4..=19).contains(&y) {
            return Ok(Self {
                ypos: y,
                block_data: [0u32; 4096],
                biome_data: [0u32; 4096],
            });
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

        let palette_blocks: Result<Vec<Block>, ChunkError> = block_states_compound
            .get_tag("palette")
            .expect_tag("no palette")?
            .get_list()
            .iter()
            .map(|f| {
                let block_compound = f.get_compound();
                let block_name = BlockName::new_from_string(
                    block_compound.get_tag("Name").unwrap().get_string().clone(),
                );

                let binding = block_compound.get_tag("Properties");
                let properties = binding.expect_tag("no palette block properties")?;

                let properties =
                    properties
                        .get_compound()
                        .iter_children()
                        .map(|(tag_name, tag)| {
                            let property_name = tag_name.as_str();

                            let property_value = tag.get_string();
                        });
                Ok(Block {
                    block_name,
                    properties: todo!(),
                })
            })
            .collect();
        let data = if palette_blocks.as_ref().unwrap().len() == 1 {
            &vec![]
        } else {
            &block_states_compound
                .get_tag("data")
                .unwrap()
                .get_long_array()
        };

        let bit_size = (f32::log2(palette_blocks.unwrap().len() as f32 - 1.0)).floor() + 1.0;

        let block_data: [u32; 4096] = std::array::from_fn(|i| {
            let ind = Self::extract_index(&data[..], i as u32, bit_size as u32);
            ind
        });

        let biome_data = [0u32; 4096];

        Ok(Self {
            ypos: y,
            block_data,
            biome_data,
        })
    }

    #[inline]
    fn extract_index(packed_array: &[i64], index: u32, bit_size: u32) -> u32 {
        if packed_array.len() == 0 {
            return 0;
        }
        let bits_per_index = std::cmp::max(bit_size, 4); // Minimum size of 4 bits
        let indices_per_element = 64 / bits_per_index; // How many indices fit into one 64-bit integer

        let element_index = index / indices_per_element;
        let within_element_index = index % indices_per_element;

        let bit_position = within_element_index * bits_per_index;

        let mask = (1 << bits_per_index) - 1;
        ((packed_array[element_index as usize] >> bit_position) & mask) as u32
    }
    #[inline(always)]
    pub fn get_block(&self, x: usize, sec_y: usize, z: usize) -> Option<&Block> {
        let num = self
            .block_data
            .get((sec_y * 16 * 16 + z * 16 + x) as usize)
            .expect("invalid index");
        //self.block_palette.get(*num)
        todo!()
    }
    fn pp(data: &[i64], x: u16, y: u16, z: u16) -> u32 {
        let bits_per_block = 4;
        let idx = Self::extract_index(data, (256 * y + 16 * z + x).into(), bits_per_block);

        data[idx as usize].try_into().unwrap()
    }
}
