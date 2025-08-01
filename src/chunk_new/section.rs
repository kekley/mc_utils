use crate::borrow::{
    nbt_compound::unaligned_types::BigEndianDouble,
    nbt_list::{CompoundList, PrimitiveList},
};

pub struct SectionTower<'a, 'rootNBT> {
    sections: CompoundList,
    min_y: usize,
    max_y: usize,
}

impl SectionTower {
    pub fn new(section_list: CompoundList) -> Self {
        todo!()
    }
}

pub struct Section {
    palette: Palette,
    data: PrimitiveList<BigEndianDouble>,
}

impl Section {
    #[inline]
    pub fn get() -> usize {}
}

pub struct Palette {
    entries: CompoundList,
}
