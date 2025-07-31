use crate::borrow::{
    nbt_compound::unaligned_types::BigEndianLong,
    nbt_list::{CompoundList, NBTList, PrimitiveList},
};

pub struct Chunk<'a, 'rootNBT> {
    x_pos: i32,
    /// Lowest y position in the chunk
    y_pos: i32,
    z_pos: i32,
    sections: SectionTower<'a, 'rootNBT>,
}

pub struct SectionTower<'a, 'rootNBT> {
    sections: CompoundList<'a, 'rootNBT>,
}
