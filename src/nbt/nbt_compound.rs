use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;
use bytes::Buf;
use bytes::Bytes;
use cesu8::from_java_cesu8;
use cesu8::to_java_cesu8;
use core::str;
use num_enum::TryFromPrimitive;
use std::borrow::Cow;
use std::io::BufRead;
use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::{fmt::Debug, sync::Arc};

use crate::resource_error::ResourceErrorKind;

use super::java_string;
use super::java_string::JavaString;
use super::nbt_error::NBTError;
use super::{
    nbt_ids::NBTId,
    nbt_tag::{get_nbt_string_bytes, NBTTag},
};

#[derive(Debug, Clone)]
pub struct NBTTagName<'a> {
    value: JavaString<'a>,
}

impl<'a> NBTTagName<'a> {
    pub fn as_str(&self) -> Cow<'_, str> {
        self.value.to_str()
    }
}

#[derive(Clone)]
pub struct NBTCompound<'a> {
    pub children: BumpVec<'a, (NBTTagName<'a>, NBTTag<'a>)>,
}
impl<'a> Debug for NBTCompound<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Compound:\n{:?}", self.children)
    }
}

impl<'a> NBTCompound<'a> {
    fn add_java_string_tag(&mut self, tag_name: JavaString<'a>, tag: NBTTag<'a>) {
        let tag_name = NBTTagName { value: tag_name };
        self.children.push((tag_name, tag));
    }
    pub fn get_tag(&self, tag_name: &str) -> Option<&NBTTag> {
        let tag = self.children.iter().find(|a| a.0.as_str() == tag_name);
        if let Some(child) = tag {
            return Some(&child.1);
        } else {
            return None;
        }
    }

    pub(crate) fn new<'b>(bytes: &mut &'a [u8], bump: &'b Bump) -> Result<Self, NBTError>
    where
        'b: 'a,
    {
        let mut tmp = NBTCompound {
            children: BumpVec::new_in(bump),
        };
        let len = bytes.len();
        let cursor = 0usize;
        while cursor < len {
            let tag_id = NBTId::try_from_primitive(bytes.get_u8())?;
            if tag_id == NBTId::EndId {
                break;
            }
            let string_bytes = get_nbt_string_bytes(bytes);
            let java_string = JavaString { data: string_bytes };
            if let Ok(tag) = NBTTag::read_tag(bytes, bump) {
                tmp.add_java_string_tag(java_string, tag);
            } else {
                break;
            }
        }
        Ok(tmp)
    }
}
