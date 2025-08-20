pub mod nbt_string {
    use std::borrow::Borrow;

    use crate::borrow::nbt_string::NBTStr;

    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    //Data is validated when converting to/from str/String
    pub struct NBTString {
        pub data: Vec<u8>,
    }

    impl NBTString {
        pub fn new_from_vec(bytes: Vec<u8>) -> Self {
            Self { data: bytes }
        }
        pub fn extend(&mut self, bytes: &[u8]) {
            self.data.extend(bytes);
        }
        pub fn new_from_str(str: &str) -> Self {
            match simd_cesu8::encode(str) {
                std::borrow::Cow::Borrowed(slice) => NBTString {
                    data: slice.to_owned(),
                },
                std::borrow::Cow::Owned(vec) => NBTString { data: vec },
            }
        }

        pub fn from_byte_slice(bytes: &[u8]) -> Self {
            NBTString {
                data: bytes.to_owned(),
            }
        }
        pub fn as_str(&self) -> &NBTStr {
            NBTStr::from_slice(self.data.as_slice())
        }
    }
    impl Borrow<NBTStr> for NBTString {
        fn borrow(&self) -> &NBTStr {
            self.as_str()
        }
    }
}

pub mod nbt_compound {

    use crate::borrow::nbt_string::NBTStr;
    use crate::nbt_error::NBTErrorKind;
    use crate::owned::nbt_string::NBTString;
    use bytes::Buf;
    use core::str;
    use num_enum::TryFromPrimitive;
    use std::fmt::Debug;
    use std::io::{Cursor, Read};
    use std::slice;

    use crate::{nbt_error::NBTError, nbt_ids::*};

    #[derive(Clone, Debug)]
    pub struct NBTCompound {
        children: Vec<(NBTString, NBTTag)>,
    }
    impl NBTCompound {
        fn add_tag(&mut self, tag_name: NBTString, tag: NBTTag) {
            self.children.push((tag_name, tag));
        }
        pub fn iter_children(&self) -> slice::Iter<'_, (NBTString, NBTTag)> {
            self.children.iter()
        }
        pub fn get_tag(&self, tag_name: &str) -> Option<&NBTTag> {
            let name = NBTStr::from_str(tag_name);
            let name = name.as_ref();

            let tag = self.children.iter().find(|a| a.0.as_str() == name);
            if let Some(child) = tag {
                Some(&child.1)
            } else {
                None
            }
        }

        pub fn from_file<T>(stream: &mut Cursor<T>) -> Result<NBTCompound, NBTError>
        where
            T: AsRef<[u8]>,
        {
            let root_id = NBTId::try_from_primitive(stream.get_u8())?;
            if root_id != NBTId::CompoundId {
                return Err(NBTError {
                    kind: NBTErrorKind::InvalidNBT("Root NBT id missing ".to_string()),
                });
            }

            let _root_name = get_nbt_string(stream)?;

            NBTCompound::new(stream)
        }
        pub fn new<T>(stream: &mut Cursor<T>) -> Result<NBTCompound, NBTError>
        where
            T: AsRef<[u8]>,
        {
            let mut tmp = NBTCompound { children: vec![] };

            loop {
                let tag = stream.get_u8();
                let id = NBTId::try_from_primitive(tag)?;
                if id == NBTId::EndId {
                    break;
                }
                let name = get_nbt_string(stream)?;

                tmp.add_tag(name, NBTTag::read_tag_payload(stream, id)?);
            }
            Ok(tmp)
        }
        pub fn pretty_print(&self, out: &mut String) {
            self.pretty_print_inner(out, 0);
        }

        fn pretty_print_inner(&self, out: &mut String, tabs: u32) {
            for _tab in 0..tabs {
                out.push('\t');
            }

            out.push_str("NBTCompound {\n");
            for (name, tag) in &self.children {
                for _tab in 0..tabs {
                    out.push('\t');
                }
                out.push_str(&name.as_str().to_str());
                out.push_str(" : ");
                if let NBTTag::Compound(nbtcompound) = tag {
                    nbtcompound.pretty_print_inner(out, tabs + 1);
                } else {
                    out.push_str(format!("{tag:?}").as_str());
                }
                out.push('\n');
            }
            for _tab in 0..tabs {
                out.push('\t');
            }
            out.push_str("}\n");
        }
    }

    #[repr(u8)]
    #[derive(Clone)]
    pub enum NBTTag {
        End = END_ID,
        Byte(i8) = BYTE_ID,
        Short(i16) = SHORT_ID,
        Int(i32) = INT_ID,
        Long(i64) = LONG_ID,
        Float(f32) = FLOAT_ID,
        Double(f64) = DOUBLE_ID,
        ByteArray(Vec<u8>) = BYTE_ARRAY_ID,
        String(NBTString) = STRING_ID,
        List(Vec<NBTTag>) = LIST_ID,
        Compound(NBTCompound) = COMPOUND_ID,
        IntArray(Vec<i32>) = INT_ARRAY_ID,
        LongArray(Vec<i64>) = LONG_ARRAY_ID,
    }

    impl Debug for NBTTag {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::End => write!(f, "End"),
                Self::Byte(arg0) => f.debug_tuple("Byte").field(arg0).finish(),
                Self::Short(arg0) => f.debug_tuple("Short").field(arg0).finish(),
                Self::Int(arg0) => f.debug_tuple("Int").field(arg0).finish(),
                Self::Long(arg0) => f.debug_tuple("Long").field(arg0).finish(),
                Self::Float(arg0) => f.debug_tuple("Float").field(arg0).finish(),
                Self::Double(arg0) => f.debug_tuple("Double").field(arg0).finish(),
                Self::ByteArray(arg0) => f.debug_tuple("ByteArray").field(arg0).finish(),
                Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
                Self::List(arg0) => f.debug_tuple("List").field(arg0).finish(),
                Self::Compound(arg0) => {
                    let mut string = String::new();
                    arg0.pretty_print(&mut string);
                    f.write_str(&string)
                }
                Self::IntArray(arg0) => f.debug_tuple("IntArray").field(arg0).finish(),
                Self::LongArray(arg0) => f.debug_tuple("LongArray").field(arg0).finish(),
            }
        }
    }

    impl NBTTag {
        /// Returns the numeric id associated with the data type.
        #[expect(unsafe_code)]
        pub const fn get_type_id(&self) -> u8 {
            // See https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
            unsafe { *(self as *const Self).cast::<u8>() }
        }
        pub fn read_tag_payload<T: AsRef<[u8]>>(
            stream: &mut Cursor<T>,
            id: NBTId,
        ) -> Result<NBTTag, NBTError> {
            match id {
                NBTId::EndId => Ok(NBTTag::End),
                NBTId::ByteId => Ok(NBTTag::Byte(stream.get_i8())),
                NBTId::ShortId => Ok(NBTTag::Short(stream.get_i16())),
                NBTId::IntId => Ok(NBTTag::Int(stream.get_i32())),
                NBTId::LongId => Ok(NBTTag::Long(stream.get_i64())),
                NBTId::FloatId => Ok(NBTTag::Float(stream.get_f32())),
                NBTId::DoubleId => Ok(NBTTag::Double(stream.get_f64())),
                NBTId::ByteArrayId => {
                    let len = stream.get_i32() as usize;
                    let mut buf = vec![0u8; len];
                    stream.read_exact(&mut buf)?;
                    Ok(NBTTag::ByteArray(buf))
                }
                NBTId::StringId => Ok(NBTTag::String(get_nbt_string(stream)?)),
                NBTId::ListId => {
                    let expected_id = NBTId::try_from_primitive(stream.get_u8())?;
                    if expected_id == NBTId::EndId {
                        return Ok(NBTTag::List(Vec::new()));
                    }
                    let len = stream.get_i32() as usize;
                    let mut list = Vec::with_capacity(len);
                    for _ in 0..len {
                        list.push(Self::read_tag_payload(stream, expected_id)?);
                    }
                    Ok(NBTTag::List(list))
                }
                NBTId::CompoundId => Ok(NBTTag::Compound(NBTCompound::new(stream)?)),
                NBTId::IntArrayId => {
                    let len = stream.get_i32() as usize;
                    let mut vec = Vec::with_capacity(len);
                    for _ in 0..len {
                        vec.push(stream.get_i32());
                    }

                    Ok(NBTTag::IntArray(vec))
                }
                NBTId::LongArrayId => {
                    //len * 8
                    //
                    //
                    let len = stream.get_i32() as usize;
                    let mut vec = Vec::with_capacity(len);
                    for _ in 0..len {
                        vec.push(stream.get_i64());
                    }

                    Ok(NBTTag::LongArray(vec))
                }
            }
        }
    }

    impl NBTTag {
        #[inline]
        pub fn get_byte(&self) -> i8 {
            if let NBTTag::Byte(value) = self {
                *value
            } else {
                panic!("Tried to read a byte from a {self:?}");
            }
        }
        #[inline]
        pub fn get_short(&self) -> i16 {
            if let NBTTag::Short(value) = self {
                *value
            } else {
                panic!("Tried to read a short from a {self:?}");
            }
        }
        #[inline]
        pub fn get_int(&self) -> i32 {
            if let NBTTag::Int(value) = self {
                *value
            } else {
                panic!("Tried to read an int from a {self:?}");
            }
        }
        #[inline]
        pub fn get_long(&self) -> i64 {
            if let NBTTag::Long(value) = self {
                *value
            } else {
                panic!("Tried to read a long from a {self:?}");
            }
        }
        #[inline]
        pub fn get_float(&self) -> f32 {
            if let NBTTag::Float(value) = self {
                *value
            } else {
                panic!("Tried to read a float from a {self:?}");
            }
        }
        #[inline]
        pub fn get_double(&self) -> f64 {
            if let NBTTag::Double(value) = self {
                *value
            } else {
                panic!("Tried to read a double from a {self:?}");
            }
        }
        #[inline]
        pub fn get_byte_array(&self) -> &[u8] {
            if let NBTTag::ByteArray(value) = self {
                value
            } else {
                panic!("Tried to read a byte array from a {self:?}");
            }
        }
        #[inline]
        pub fn get_string(&self) -> &NBTString {
            if let NBTTag::String(value) = self {
                value
            } else {
                panic!("Tried to read a string from a {self:?}");
            }
        }
        #[inline]
        pub fn get_list(&self) -> &[NBTTag] {
            if let NBTTag::List(value) = self {
                value
            } else {
                panic!("Tried to read a list from a {self:?}");
            }
        }
        #[inline]
        pub fn get_compound(&self) -> &NBTCompound {
            if let NBTTag::Compound(value) = self {
                value
            } else {
                panic!("Tried to read a compound from a {self:?}");
            }
        }
        #[inline]
        pub fn get_int_array(&self) -> &Vec<i32> {
            if let NBTTag::IntArray(value) = self {
                value
            } else {
                panic!("Tried to read an int array from a {self:?}");
            }
        }
        #[inline]
        pub fn get_long_array(&self) -> &Vec<i64> {
            if let NBTTag::LongArray(value) = self {
                value
            } else {
                panic!("Tried to read a long array from a {self:?}");
            }
        }
    }

    #[inline]
    pub fn get_nbt_string<T: AsRef<[u8]>>(stream: &mut Cursor<T>) -> Result<NBTString, NBTError> {
        let len = stream.get_u16() as usize;
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf)?;
        Ok(NBTString::new_from_vec(buf))
    }
}

#[cfg(test)]
mod nbt_test {
    use std::io::Cursor;

    use super::nbt_compound::NBTCompound;

    #[test]
    fn test_file() {
        let path = "./test_assets/iceandfire_myrmex.dat";
        let level_dat = std::fs::read(path).unwrap_or_else(|_| panic!("could not find {path}"));
        let mut cursor = Cursor::new(level_dat);
        let compound = NBTCompound::from_file(&mut cursor).expect("NBT parse error");
        let mut string = String::new();
        compound.pretty_print(&mut string);
    }
}
