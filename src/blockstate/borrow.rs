use std::fmt::{Debug, Display};

use cesu8_str::java::JavaStr;

use crate::{
    borrow::{
        nbt_compound::{NBTCompound, NBTCompoundIter},
        nbt_string::NBTStr,
    },
    owned::nbt_string::NBTString,
};

use super::BlockStateTrait;

#[derive(Clone)]
pub struct BlockState<'data, 'root_nbt> {
    name: &'data NBTStr,
    properties: Option<NBTCompound<'data, 'root_nbt>>,
}

const AIR_BLOCK_NAME: &NBTStr = const { NBTStr::from_slice(b"minecraft:air") };
const WATERLOGGED_NAME: &NBTStr = const { NBTStr::from_slice(b"waterlogged") };
const TRUE_VALUE: &NBTStr = const { NBTStr::from_slice(b"true") };

impl<'data, 'root_nbt> BlockState<'data, 'root_nbt> {
    pub fn get_name(&self) -> &NBTStr {
        self.name
    }
    pub fn is_waterlogged(&self) -> bool {
        self.properties_iter()
            .any(|(name, value)| name == WATERLOGGED_NAME && value == TRUE_VALUE)
    }
    pub fn to_mapped_state(&self) -> NBTString {
        let mut vec: Vec<u8> = Vec::new();
        self.write_mapped_state(&mut vec);
        NBTString::new_from_vec(vec)
    }
    ///A mapped state follows the format ``namespace:block_name#prop1=value,prop2=value`` ...
    ///If there are no properties, the property string is just "default" as in
    ///minecraft:air#default
    #[expect(unsafe_code)]
    pub fn write_mapped_state(&self, mut out: impl std::io::Write) {
        let block_name = self.name;
        let _ = out.write_all(block_name.as_bytes());
        let _ = out.write_all(b"#");
        let mut i = 0;
        let mut to_sort: Vec<_> = self
            .properties_iter()
            .filter(|(name, _value)| *name != WATERLOGGED_NAME)
            .collect();

        to_sort.sort_by(|a, b| {
            let a = unsafe { JavaStr::from_java_cesu8_unchecked(a.0.as_bytes()) };
            let b = unsafe { JavaStr::from_java_cesu8_unchecked(b.0.as_bytes()) };

            a.cmp(b)
        });

        for (name, value) in to_sort {
            if i > 0 {
                let _ = out.write_all(b",");
            }

            name.write_lowercase(&mut out);

            let _ = out.write_all(b"=");

            value.write_lowercase(&mut out);
            i += 1;
        }
        if i == 0 {
            let _ = out.write_all(b"normal");
        }
    }

    pub fn properties_iter(&self) -> PropertiesIter<'_, '_> {
        PropertiesIter {
            compound_iter: self.properties.as_ref().map(|compound| compound.iter()),
        }
    }
    pub fn from_compound(compound: NBTCompound<'data, 'root_nbt>) -> Option<Self> {
        let name_tag = compound.get_tag("Name")?;
        let name = name_tag.get_string()?;

        let properties = compound
            .get_tag("Properties")
            .and_then(|tag| tag.get_compound());

        Some(Self { name, properties })
    }
}

impl Debug for BlockState<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self))
    }
}

impl Display for BlockState<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_mapped_state().as_nbt_str().to_str())
    }
}

pub struct PropertiesIter<'data, 'root_nbt> {
    compound_iter: Option<NBTCompoundIter<'data, 'root_nbt>>,
}

impl<'data> Iterator for PropertiesIter<'data, '_> {
    type Item = (&'data NBTStr, &'data NBTStr);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(iter) = &mut self.compound_iter {
            for (name, tag) in iter.by_ref() {
                if let Some(value) = tag.get_string() {
                    return Some((name, value));
                }
            }
        }
        None
    }
}

impl<'data, 'root_nbt> BlockStateTrait for BlockState<'data, 'root_nbt> {
    fn name(&self) -> &'data NBTStr {
        self.name
    }

    #[allow(refining_impl_trait)]
    fn iter_properties(&self) -> impl Iterator<Item = (&NBTStr, &NBTStr)> {
        self.properties_iter()
    }
}
