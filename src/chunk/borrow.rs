use crate::{
    borrow::{nbt_compound::RootNBTCompound, nbt_list::ListType},
    section::borrow::{PackingType, SectionTower},
};

pub struct Chunk<'a> {
    compound: RootNBTCompound<'a>,
    packing_type: PackingType,
    x_pos: i32,
    y_pos: i32,
    z_pos: i32,
}

impl<'a> Chunk<'a> {
    pub fn from_compound(compound: RootNBTCompound<'a>) -> Option<Self> {
        let version_tag = compound.get_tag("DataVersion")?;
        let x_tag = compound.get_tag("xPos")?;
        let y_tag = compound.get_tag("yPos")?;
        let z_tag = compound.get_tag("zPos")?;

        let x_pos = x_tag.get_int()?;
        let y_pos = y_tag.get_int()?;
        let z_pos = z_tag.get_int()?;
        let version = version_tag.get_int()?;

        let packing_type = if version >= 2556 {
            PackingType::Post1_16
        } else {
            PackingType::Pre1_16
        };

        Some(Self {
            compound,
            packing_type,
            x_pos,
            y_pos,
            z_pos,
        })
    }
    pub fn get_x(&self) -> isize {
        self.x_pos as isize
    }

    pub fn get_z(&self) -> isize {
        self.z_pos as isize
    }

    pub fn get_sections<'root_nbt>(&'root_nbt self) -> Option<SectionTower<'a, 'root_nbt>> {
        let tag = self.compound.get_tag("sections")?;

        let list = tag.get_list()?;

        if let ListType::Compound(compound_list) = list {
            Some(SectionTower::from_compound_list(
                compound_list,
                self.packing_type,
            ))
        } else {
            None
        }
    }
}
