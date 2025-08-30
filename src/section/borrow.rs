use crate::blockstate::borrow::BlockState;
use crate::borrow::nbt_compound::unaligned_types::BigEndianLong;
use crate::borrow::nbt_compound::NBTCompound;
use crate::borrow::nbt_compound::NBTCompoundIter;
use crate::borrow::nbt_list::CompoundList;
use crate::borrow::nbt_list::CompoundListIter;
use crate::borrow::nbt_list::ListType;
use crate::nbt::borrow::nbt_compound::unaligned_types::UnalignedType;

pub struct SectionTower<'a, 'root_nbt> {
    sections: CompoundList<'a, 'root_nbt>,
    packing_type: PackingType,
}

impl<'data, 'root_nbt> SectionTower<'data, 'root_nbt> {
    pub fn from_compound_list(
        section_list: CompoundList<'data, 'root_nbt>,
        packing_type: PackingType,
    ) -> Self {
        Self {
            sections: section_list,
            packing_type,
        }
    }
    #[inline]
    fn y_to_section_index(y: isize) -> isize {
        y / 16
            + if y % 16 == 0 {
                0
            } else if y > 0 {
                1
            } else {
                -1
            }
    }
    pub fn get_section_for_y(&self, y: isize) -> Option<Section<'data, 'root_nbt>> {
        let y = Self::y_to_section_index(y);

        let compound = self.sections.iter().find(|section_compound| {
            let tag = section_compound.get_tag("Y");
            if let Some(tag) = tag {
                if let Some(section_y) = tag.get_byte() {
                    section_y as isize == y
                } else if let Some(section_y) = tag.get_int() {
                    section_y as isize == y
                } else {
                    false
                }
            } else {
                false
            }
        });
        if let Some(compound) = compound {
            Some(Section::from_compound(compound, self.packing_type)?)
        } else {
            None
        }
    }

    pub fn iter_sections(&self) -> SectionTowerIter<'data, 'root_nbt> {
        SectionTowerIter {
            iter: self.sections.iter().clone(),
            packing_type: self.packing_type,
        }
    }
}

pub struct SectionTowerIter<'a, 'root_nbt> {
    iter: CompoundListIter<'a, 'root_nbt>,
    packing_type: PackingType,
}

impl<'data, 'root_nbt> Iterator for SectionTowerIter<'data, 'root_nbt> {
    type Item = Section<'data, 'root_nbt>;

    fn next(&mut self) -> Option<Self::Item> {
        for compound in &mut self.iter {
            if let Some(section) = Section::from_compound(compound, self.packing_type) {
                return Some(section);
            }
        }
        None
    }
}

pub struct Section<'a, 'root_nbt> {
    palette: Palette<'a, 'root_nbt>,
    packing_type: PackingType,
    y_index: i8,
    bits_per_index: u8,
    data: Option<&'a [BigEndianLong]>,
}

pub struct SectionIndexIter<'a> {
    data: Option<&'a [BigEndianLong]>,
    bits_per_index: u8,
    packing_type: PackingType,
    index: usize,
}

impl<'a> Iterator for SectionIndexIter<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= 4096 {
            return None;
        }
        let index = if let Some(data) = self.data {
            match self.packing_type {
                PackingType::Pre1_16 => {
                    read_packed_index_pre116(data, self.index, self.bits_per_index)?
                }
                PackingType::Post1_16 => {
                    read_packed_index_post116(data, self.index, self.bits_per_index)?
                }
            }
        } else {
            0
        };
        self.index += 1;
        Some(index)
    }
}

impl<'data, 'root_nbt> Section<'data, 'root_nbt> {
    pub fn from_compound(
        compound: NBTCompound<'data, 'root_nbt>,
        packing_type: PackingType,
    ) -> Option<Self> {
        let block_states_tag = compound.get_tag("block_states")?;
        let block_states_compound = block_states_tag.get_compound()?;
        let palette_tag = block_states_compound.get_tag("palette")?;
        let palette_list = palette_tag.get_list()?;

        let palette_compound_list = if let ListType::Compound(compound_list) = palette_list {
            compound_list
        } else {
            return None;
        };
        let palette = Palette::from_compound_list(palette_compound_list);
        let palette_length = palette.iter().count();
        let bits_per_index = ((palette_length as f32).log2().ceil() as u32).max(4);

        let data = if let Some(tag) = block_states_compound.get_tag("data") {
            tag.get_long_array()
        } else {
            None
        };

        let y_index_tag = compound.get_tag("Y")?;

        let y_index = if let Some(y) = y_index_tag.get_byte() {
            y
        } else if let Some(y) = y_index_tag.get_int() {
            y as i8
        } else {
            return None;
        };

        Some(Self {
            palette,
            bits_per_index: bits_per_index as u8,
            data,
            y_index,
            packing_type,
        })
    }

    pub fn get_palette(&self) -> Palette<'data, 'root_nbt> {
        self.palette.clone()
    }

    pub fn iter_block_indices(&self) -> SectionIndexIter<'data> {
        let Self {
            palette: _,
            y_index: _,
            bits_per_index,
            data,
            packing_type,
        } = self;
        SectionIndexIter {
            data: *data,
            bits_per_index: *bits_per_index,
            packing_type: *packing_type,
            index: 0,
        }
    }
    /// Gets the lowest Y block in the section
    pub fn get_lowest_y(&self) -> isize {
        self.y_index as isize * 16
    }
    pub fn get_y_index(&self) -> i8 {
        self.y_index
    }
    #[inline]
    fn get_block_index(&self, x: u8, y: u8, z: u8) -> u16 {
        let x = x as usize;
        let y = y as usize;
        let z = z as usize;
        let index = y * 16 * 16 + z * 16 + x;
        if let Some(data) = self.data {
            match self.packing_type {
                PackingType::Pre1_16 => read_packed_index_pre116(data, index, self.bits_per_index)
                    .expect("Index should be between 0 and 4095"),
                PackingType::Post1_16 => {
                    read_packed_index_post116(data, index, self.bits_per_index)
                        .expect("Index should be between 0 and 4095")
                }
            }
        } else {
            0
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PackingType {
    Pre1_16,
    Post1_16,
}

#[derive(Clone, Debug)]
pub struct Palette<'a, 'root_nbt> {
    entries: CompoundListIter<'a, 'root_nbt>,
}

impl<'a, 'root_nbt> Palette<'a, 'root_nbt> {
    pub fn from_compound_list(palette_compound_list: CompoundList<'a, 'root_nbt>) -> Self {
        Self {
            entries: palette_compound_list.iter(),
        }
    }
    pub fn iter(&self) -> PaletteIter {
        PaletteIter {
            iter: self.entries.clone(),
        }
    }
}

pub struct PaletteIter<'a, 'root_nbt> {
    iter: CompoundListIter<'a, 'root_nbt>,
}

impl<'data, 'root_nbt> Iterator for PaletteIter<'data, 'root_nbt> {
    type Item = BlockState<'data, 'root_nbt>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(compound) = self.iter.next() {
            if let Some(blockstate) = BlockState::from_compound(compound) {
                return Some(blockstate);
            }
        }
        None
    }
}

#[inline]
fn read_packed_index_pre116(
    data: &[BigEndianLong],
    index: usize,
    bits_per_index: u8,
) -> Option<u16> {
    println!("pre");
    let bits_per_index = bits_per_index as usize;
    let bit_mask: u64 = u64::MAX.unbounded_shr(u64::BITS - bits_per_index as u32);

    let bit_index = index * bits_per_index;

    let long_index = bit_index / u64::BITS as usize;

    let bit_offset_within_long = bit_index % u64::BITS as usize;

    let lower_bytes = data.get(long_index)?.to_aligned_ne();

    let upper_bytes = data
        .get(long_index + 1)
        .map_or(0_i64, |word| word.to_aligned_ne());

    let double_long = ((upper_bytes.cast_unsigned() as u128).unbounded_shl(64))
        | (lower_bytes.cast_unsigned() as u128);

    let shifted = (double_long.unbounded_shr(bit_offset_within_long as u32)) as u64;

    Some((shifted & bit_mask) as u16)
}
#[inline]
fn read_packed_index_post116(
    data: &[BigEndianLong],
    index: usize,
    bits_per_index: u8,
) -> Option<u16> {
    let values_per_long = u64::BITS / bits_per_index as u32;
    let long_index = index / values_per_long as usize;
    let shift_right_amount: u32 = bits_per_index as u32 * (index as u32 % values_per_long);
    let shift_left_amount: u32 = u64::BITS - bits_per_index as u32 - shift_right_amount;

    let long = data.get(long_index)?.to_aligned_ne();

    let shifted = (long.cast_unsigned().unbounded_shl(shift_left_amount))
        .unbounded_shr(shift_right_amount + shift_left_amount);

    Some(shifted as u16)
}

#[cfg(test)]
mod test {
    use std::{array, time::Instant};

    use super::*;
    #[test]
    fn packed_indices_pre116() {
        const ARRAY_SIZE: usize = 4096;
        let start_time = Instant::now();
        for i in 1..=8 {
            let test_array: [u8; ARRAY_SIZE] =
                array::from_fn(|_| fastrand::u8(..(2u8.saturating_pow(i))));
            let mut dest = [BigEndianLong::from_ne(0); ARRAY_SIZE / 8];

            let max = *test_array.iter().max().unwrap();

            let bits_per_index = ((max as f32).log2().ceil() as u8).max(4);

            write_packed_indices_pre116(&test_array, &mut dest, bits_per_index);

            for (i, element) in test_array.iter().enumerate() {
                let value = read_packed_index_pre116(&dest, i, bits_per_index).unwrap();

                assert_eq!(*element as u16, value);
            }
        }
        let duration = Instant::now().duration_since(start_time);

        println!("Duration: {duration:?}");
    }

    #[test]
    fn packed_indices_post116() {
        const ARRAY_SIZE: usize = 4096;
        let start_time = Instant::now();
        for i in 1..=8 {
            let test_array: [u8; ARRAY_SIZE] =
                array::from_fn(|_| fastrand::u8(..(2u8.saturating_pow(i))));
            let mut dest = [BigEndianLong::from_ne(0); ARRAY_SIZE / 4];

            let max = *test_array.iter().max().unwrap();

            let bits_per_index = ((max as f32).log2().ceil() as u8).max(4);

            write_packed_indices_post116(&test_array, &mut dest, bits_per_index);

            for (i, element) in test_array.iter().enumerate() {
                let value = read_packed_index_post116(&dest, i, bits_per_index).unwrap();

                assert_eq!(*element as u16, value);
            }
        }
        let duration = Instant::now().duration_since(start_time);

        println!("Duration: {duration:?}");
    }
    fn write_packed_indices_pre116(indices: &[u8], dest: &mut [BigEndianLong], bits_per_index: u8) {
        for (index, value) in indices.iter().enumerate() {
            let bit_index = index * bits_per_index as usize;
            let long_index = bit_index / u64::BITS as usize;
            let bit_offset = bit_index % u64::BITS as usize;
            let double_long = (*value as u128).unbounded_shl(bit_offset as u32);

            if let Some(lower_word) = dest.get_mut(long_index) {
                *lower_word |= BigEndianLong::from_ne((double_long as u64).cast_signed());
            }

            if let Some(upper_word) = dest.get_mut(long_index + 1) {
                *upper_word |=
                    BigEndianLong::from_ne(((double_long.unbounded_shr(64)) as u64).cast_signed());
            }
        }
    }

    fn write_packed_indices_post116(values: &[u8], dest: &mut [BigEndianLong], bits_per_index: u8) {
        let values_per_long = u64::BITS / bits_per_index as u32;

        for (index, value) in values.iter().enumerate() {
            let long_index = index / values_per_long as usize;
            let shift_amount = bits_per_index as usize * (index % values_per_long as usize);

            let shifted = (*value as u64) << shift_amount;

            if let Some(long) = dest.get_mut(long_index) {
                *long |= BigEndianLong::from_ne((shifted).cast_signed());
            }
        }
    }
}
