#[allow(dead_code)]
#[allow(clippy::unimplemented)]
#[allow(clippy::todo)]
#[allow(unsafe_code)]
pub mod nbt_compound {

    use std::{io::Cursor, marker::PhantomData};

    use crate::borrow::list::ParseStackEntry;
    use crate::borrow::list::ParsingStack;
    use crate::{nbt_error::NBTError, nbt_ids::*};
    use byteorder::{BigEndian, ReadBytesExt};
    use bytes::Buf;
    use num_enum::TryFromPrimitive;

    use super::list::ParsingError;
    use super::nbt_string::NBTStr;

    pub struct NBTCompoundIter<'a, 'root> {
        current_offset: usize,
        max_offset: usize,
        data: &'a [u8],
        elements: &'root [Element],
        inner_elements: &'root [InnerElement],
    }
    impl<'a, 'root> Iterator for NBTCompoundIter<'a, 'root> {
        type Item = (&'a NBTStr, NBTTag<'a, 'root>);

        fn next(&mut self) -> Option<Self::Item> {
            if self.current_offset + 1 >= self.max_offset {
                return None;
            }

            let name_element = self.elements[self.current_offset];
            assert!(name_element.get_id() == ElementTag::NameString);
            let offset = name_element.get_offset() as usize;
            let string_len = u16::from_be_bytes(
                self.data[offset..offset + 2]
                    .try_into()
                    .expect("This should always be a length of two"),
            ) as usize;
            let string_start = offset + size_of::<i16>();
            let string_end = string_start + string_len;
            let string_slice = &self.data[string_start..string_end];

            let str = NBTStr::from_slice(string_slice).to_str();

            println!("{str}");
            self.current_offset += 1;

            let element = self.elements[self.current_offset];

            self.current_offset += element.get_skip_amount();

            todo!()
        }
    }

    pub struct NBTTag<'a, 'root> {
        data: &'a [u8],
        this_element: Element,
        inner_elements: &'root [InnerElement],
    }

    pub struct RootNBTCompound<'a> {
        name: &'a NBTStr,
        elements: Vec<Element>,
        inner_elements: Vec<InnerElement>,
        data: &'a [u8],
    }

    impl<'a> RootNBTCompound<'a> {
        fn read_tag_id(cursor: &mut Cursor<&[u8]>) -> Result<NBTId, NBTError> {
            let u8 = cursor.read_u8()?;
            Ok(NBTId::try_from_primitive(u8)?)
        }
        fn read_u16_length_and_slice_of(
            cursor: &mut Cursor<&[u8]>,
            data_width: usize,
        ) -> Result<(), NBTError> {
            let length = cursor.read_u16::<BigEndian>()?;
            let length = length as usize * data_width;
            assert!(cursor.remaining() >= length);
            cursor.advance(length);
            Ok(())
        }
        fn read_u32_length_and_slice_of(
            cursor: &mut Cursor<&[u8]>,
            data_width: usize,
        ) -> Result<(), NBTError> {
            let length = cursor.read_u32::<BigEndian>()?;
            let length = length as usize * data_width;
            assert!(cursor.remaining() >= (length));
            cursor.advance(length);
            Ok(())
        }

        fn take_nbt_string(cursor: &mut Cursor<&'a [u8]>) -> Result<&'a NBTStr, NBTError> {
            let len = cursor.read_u16::<BigEndian>()?;
            let start = cursor.position() as usize;
            let end = start + len as usize;
            let slice = &cursor.get_ref()[start..end];
            Ok(NBTStr::from_slice(slice))
        }

        pub fn from_file(bytes: &'a [u8]) -> Result<Self, NBTError> {
            assert!(bytes.len() as u64 <= 0x00FF_FFFF_FFFF_FFFF);
            let slice = bytes;
            let mut cursor = Cursor::new(slice);
            let root_id = Self::read_tag_id(&mut cursor)?;

            if root_id != NBTId::CompoundId {
                return Err(NBTError {
                    kind: crate::nbt_error::NBTErrorKind::InvalidNBT(
                        "File is missing root compound".to_string(),
                    ),
                });
            }
            let root_name = Self::take_nbt_string(&mut cursor)?;
            let mut elements = Vec::new();
            let mut inner_elements = Vec::new();
            let mut parsing_stack = ParsingStack::default();
            parsing_stack.push(ParseStackEntry::compound(0))?;
            elements.push(Element::from_compound());

            while !parsing_stack.is_empty() {
                match parsing_stack.peek()?.ty() {
                    crate::borrow::list::CringeCompoundTypes::Compound => {
                        Self::read_tag_in_compound(
                            &mut cursor,
                            &mut elements,
                            &mut inner_elements,
                            &mut parsing_stack,
                        )?;
                    }
                    crate::borrow::list::CringeCompoundTypes::ListOfCompounds => {
                        Self::read_compound_in_list(
                            &mut cursor,
                            &mut elements,
                            &mut inner_elements,
                            &mut parsing_stack,
                        )?;
                    }
                    crate::borrow::list::CringeCompoundTypes::ListOfLists => {
                        Self::read_list_in_list(
                            &mut cursor,
                            &mut elements,
                            &mut inner_elements,
                            &mut parsing_stack,
                        )?;
                    }
                }
            }
            Ok(RootNBTCompound {
                name: root_name,
                elements,
                inner_elements: vec![],
                data: bytes,
            })
        }
        fn iter<'root>(&'root self) -> NBTCompoundIter<'a, 'root> {}
        fn read_tag_in_compound(
            cursor: &mut Cursor<&[u8]>,
            elements: &mut Vec<Element>,
            inner_elements: &mut Vec<InnerElement>,
            stack: &mut ParsingStack,
        ) -> Result<(), NBTError> {
            let tag_id = Self::read_tag_id(cursor)?;
            let name_element = Element::name_string(cursor.position());
            elements.push(name_element);

            let element = match tag_id {
                NBTId::EndId => {
                    Self::handle_compound_end(elements, stack)?;
                    return Ok(());
                }
                NBTId::ByteId => {
                    let value = cursor.read_u8()?;
                    Element::from_byte(value)
                }
                NBTId::ShortId => {
                    let value = cursor.read_u16::<BigEndian>()?;
                    Element::from_short(value)
                }
                NBTId::IntId => {
                    let value = cursor.read_u32::<BigEndian>()?;
                    Element::from_int(value)
                }
                NBTId::LongId => {
                    let offset = cursor.position();
                    //we need to advance the cursor since we just stored an offset
                    cursor.advance(8);
                    Element::from_long(offset)
                }
                NBTId::FloatId => {
                    let value = cursor.read_u32::<BigEndian>()?;
                    Element::from_float(value)
                }
                NBTId::DoubleId => {
                    let offset = cursor.position();

                    //we need to advance the cursor since we just stored an offset
                    cursor.advance(8);
                    Element::from_double(offset)
                }
                NBTId::ByteArrayId => {
                    let offset = cursor.position();
                    Self::read_u32_length_and_slice_of(cursor, 1)?;
                    Element::from_byte_array(offset)
                }
                NBTId::StringId => {
                    let offset = cursor.position();
                    Self::read_u16_length_and_slice_of(cursor, 1)?;
                    Element::from_string(offset)
                }
                NBTId::LongArrayId => {
                    let offset = cursor.position();
                    Self::read_u32_length_and_slice_of(cursor, 8)?;
                    Element::from_long_array(offset)
                }
                NBTId::IntArrayId => {
                    let offset = cursor.position();
                    Self::read_u32_length_and_slice_of(cursor, 4)?;
                    Element::from_int_array(offset)
                }
                // these suck
                NBTId::ListId => {
                    return Self::list(cursor, elements, inner_elements, stack);
                }
                NBTId::CompoundId => {
                    return Self::compound(cursor, elements, inner_elements, stack);
                }
            };
            elements.push(element);
            Ok(())
        }
        fn list(
            cursor: &mut Cursor<&[u8]>,
            elements: &mut Vec<Element>,
            inner_elements: &mut Vec<InnerElement>,
            stack: &mut ParsingStack,
        ) -> Result<(), NBTError> {
            let list_tag_id = Self::read_tag_id(cursor)?;
            match list_tag_id {
                NBTId::EndId => {
                    assert!(cursor.remaining() >= 4);
                    cursor.advance(4);
                }
                NBTId::ByteId => {
                    let offset = cursor.position();
                    let size_of_byte = 1;
                    elements.push(Element::list_of_bytes(offset));
                    Self::read_u32_length_and_slice_of(cursor, size_of_byte)?;
                }
                NBTId::ShortId => {
                    let offset = cursor.position();
                    let size_of_short = 2;
                    elements.push(Element::list_of_shorts(offset));
                    Self::read_u32_length_and_slice_of(cursor, size_of_short)?;
                }
                NBTId::IntId => {
                    let offset = cursor.position();
                    let size_of_int = 4;
                    elements.push(Element::list_of_ints(offset));
                    Self::read_u32_length_and_slice_of(cursor, size_of_int)?;
                }
                NBTId::LongId => {
                    let offset = cursor.position();
                    let size_of_long = 8;
                    elements.push(Element::list_of_longs(offset));
                    Self::read_u32_length_and_slice_of(cursor, size_of_long)?;
                }
                NBTId::FloatId => {
                    let offset = cursor.position();
                    let size_of_float = 4;
                    elements.push(Element::list_of_floats(offset));
                    Self::read_u32_length_and_slice_of(cursor, size_of_float)?;
                }
                NBTId::DoubleId => {
                    let offset = cursor.position();
                    let size_of_double = 8;
                    elements.push(Element::list_of_doubles(offset));
                    Self::read_u32_length_and_slice_of(cursor, size_of_double)?;
                }
                NBTId::ByteArrayId => {
                    let index_of_inner = inner_elements.len() as u32;
                    let length = cursor.read_u32::<BigEndian>()?;
                    inner_elements.push(InnerElement::Length(length));
                    let byte_size = 1;
                    for _ in 0..length {
                        let offset = cursor.position();
                        Self::read_u32_length_and_slice_of(cursor, byte_size)?;
                        inner_elements.push(InnerElement::ByteArray(offset));
                    }
                    elements.push(Element::list_of_byte_arrays(index_of_inner));
                }
                NBTId::StringId => {
                    let index_of_inner = inner_elements.len() as u32;
                    let length = cursor.read_u32::<BigEndian>()?;
                    inner_elements.push(InnerElement::Length(length));
                    let byte_size = 1;
                    for _ in 0..length {
                        let offset = cursor.position();
                        Self::read_u16_length_and_slice_of(cursor, byte_size)?;
                        inner_elements.push(InnerElement::String(offset));
                    }
                    elements.push(Element::list_of_strings(index_of_inner));
                }
                NBTId::ListId => {
                    let length = cursor.read_u32::<BigEndian>()?;

                    let index_of_element = elements.len() as u32;

                    stack.push(ParseStackEntry::list_of_lists(index_of_element))?;

                    stack.set_list_length(length)?;

                    elements.push(Element::list_of_lists(length));
                }
                NBTId::CompoundId => {
                    let length = cursor.read_u32::<BigEndian>()?;

                    let index_of_element = elements.len() as u32;

                    stack.push(ParseStackEntry::list_of_compounds(index_of_element))?;
                    stack.set_list_length(length)?;

                    elements.push(Element::list_of_compounds(length));
                }
                NBTId::IntArrayId => {
                    let index_of_inner = inner_elements.len() as u32;
                    let length = cursor.read_u32::<BigEndian>()?;
                    inner_elements.push(InnerElement::Length(length));
                    let int_size = 4;
                    for _ in 0..length {
                        let offset = cursor.position();
                        Self::read_u32_length_and_slice_of(cursor, int_size)?;
                        inner_elements.push(InnerElement::IntArray(offset));
                    }
                    elements.push(Element::list_of_int_arrays(index_of_inner));
                }
                NBTId::LongArrayId => {
                    let index_of_inner = inner_elements.len() as u32;
                    let length = cursor.read_u32::<BigEndian>()?;
                    inner_elements.push(InnerElement::Length(length));
                    let long_size = 8;
                    for _ in 0..length {
                        let offset = cursor.position();
                        Self::read_u32_length_and_slice_of(cursor, long_size)?;
                        inner_elements.push(InnerElement::LongArray(offset));
                    }
                    elements.push(Element::list_of_long_arrays(index_of_inner));
                }
            }
            Ok(())
        }

        fn compound(
            cursor: &mut Cursor<&[u8]>,
            elements: &mut Vec<Element>,
            inner_elements: &mut Vec<InnerElement>,
            stack: &mut ParsingStack,
        ) -> Result<(), NBTError> {
            let compound_start_index = elements.len();

            stack.push(ParseStackEntry::compound(compound_start_index as u32))?;

            elements.push(Element::from_compound());

            Ok(())
        }

        fn read_compound_in_list(
            cursor: &mut Cursor<&[u8]>,
            elements: &mut Vec<Element>,
            inner_elements: &mut Vec<InnerElement>,
            parsing_stack: &mut ParsingStack,
        ) -> Result<(), NBTError> {
            let list_index = parsing_stack.peek()?.index();

            let elements_remaining = parsing_stack.remaining_elements_in_list()?;

            if elements_remaining == 0 {
                parsing_stack.pop()?;

                let index_after_end = elements.len();

                elements
                    .get_mut(list_index)
                    .expect("list element index should be valid")
                    .set_offset((index_after_end - list_index) as u32);
                return Ok(());
            }
            parsing_stack.decrement_list_length()?;
            Self::compound(cursor, elements, inner_elements, parsing_stack)
        }

        fn read_list_in_list(
            cursor: &mut Cursor<&[u8]>,
            elements: &mut Vec<Element>,
            inner_elements: &mut Vec<InnerElement>,
            parsing_stack: &mut ParsingStack,
        ) -> Result<(), NBTError> {
            let list_index = parsing_stack.peek()?.index();
            let remaining = parsing_stack.remaining_elements_in_list()?;

            if remaining == 0 {
                parsing_stack.pop()?;

                let index_after_end = elements.len();
                elements
                    .get_mut(list_index)
                    .expect("List index should be valid")
                    .set_offset((index_after_end - list_index) as u32);
                return Ok(());
            }
            parsing_stack.decrement_list_length()?;

            Self::list(cursor, elements, inner_elements, parsing_stack)
        }
        fn handle_compound_end(
            elements: &mut Vec<Element>,
            parsing_stack: &mut ParsingStack,
        ) -> Result<(), ParsingError> {
            let compound_start_index = parsing_stack.pop()?.index();
            let compound_end_index = elements.len();

            match elements.get_mut(compound_start_index) {
                Some(element) => {
                    element.set_offset((compound_end_index - compound_start_index) as u32);
                }
                None => return Err(ParsingError::INTERNAL),
            }

            Ok(())
        }
    }

    pub mod unaligned_types {

        #[derive(Debug, Clone, Copy)]
        #[repr(C, packed)]
        pub struct Long(pub u64);

        #[derive(Debug, Clone, Copy)]
        #[repr(C, packed)]
        pub struct Double(pub u64);

        #[derive(Debug, Clone, Copy)]
        #[repr(C, packed)]
        pub struct Int(pub u32);

        #[derive(Debug, Clone, Copy)]
        #[repr(C, packed)]
        pub struct Short(pub u16);
    }

    ///bit pattern: u8 | u24 | u32
    ///           kind | len | offset
    #[derive(Clone, Copy)]
    struct Element(u64);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
    #[repr(u8)]
    pub enum ElementTag {
        EmptyList = 0,
        Byte = BYTE_ID,
        Short = SHORT_ID,
        Int = INT_ID,
        Long = LONG_ID,
        Float = FLOAT_ID,
        Double = DOUBLE_ID,
        ByteArray = BYTE_ARRAY_ID,
        String = STRING_ID,
        Compound = COMPOUND_ID,
        IntArray = INT_ARRAY_ID,
        LongArray = LONG_ARRAY_ID,
        NameString,
        ByteList,
        ShortList,
        IntList,
        LongList,
        FloatList,
        DoubleList,
        ByteArrayList,
        StringList,
        ListList,
        CompoundList,
        IntArrayList,
        LongArrayList,
    }

    impl Element {
        pub fn get_id(self) -> ElementTag {
            let u8 = (self.0 >> 56) as u8;
            ElementTag::try_from_primitive(u8).unwrap()
        }
        pub fn get_truncated_len_and_offset(self) -> (u32, u32) {
            let trunc_len = ((self.0 >> 32) & 0xFF_FFFF) as u32;
            let offset = self.0 as u32;
            (trunc_len, offset)
        }

        pub fn get_offset(self) -> u32 {
            self.0 as u32
        }
        pub fn get_skip_amount(self) -> usize {
            match self.get_id() {
                ElementTag::Compound => todo!(),
                ElementTag::ListList => todo!(),
                ElementTag::CompoundList => todo!(),
                _ => 1,
            }
        }
    }

    impl Element {
        pub fn from_compound() -> Self {
            let value = (COMPOUND_ID as u64) << 56;
            Self(value)
        }
        pub fn from_byte(byte: u8) -> Self {
            let value = ((BYTE_ID as u64) << 56) | byte as u64;
            Self(value)
        }

        pub fn from_short(short: u16) -> Self {
            let value = ((SHORT_ID as u64) << 56) | short as u64;
            Self(value)
        }

        pub fn from_int(int: u32) -> Self {
            let value = ((INT_ID as u64) << 56) | int as u64;
            Self(value)
        }
        pub fn from_long(offset: u64) -> Self {
            let value = ((LONG_ID as u64) << 56) | offset;
            Self(value)
        }

        pub fn from_float(float: u32) -> Self {
            let value = ((FLOAT_ID as u64) << 56) | float as u64;
            Self(value)
        }

        pub fn from_double(offset: u64) -> Self {
            let value = ((DOUBLE_ID as u64) << 56) | offset;
            Self(value)
        }
        pub fn from_byte_array(offset: u64) -> Self {
            let value = ((BYTE_ARRAY_ID as u64) << 56) | offset;
            Self(value)
        }

        pub fn from_string(offset: u64) -> Self {
            let value = ((STRING_ID as u64) << 56) | offset;
            Self(value)
        }

        pub fn from_long_array(offset: u64) -> Self {
            let value = ((LONG_ARRAY_ID as u64) << 56) | offset;
            Self(value)
        }
        fn from_int_array(offset: u64) -> Element {
            let value = ((INT_ARRAY_ID as u64) << 56) | offset;
            Self(value)
        }

        pub fn name_string(offset: u64) -> Self {
            let value = ((ElementTag::NameString as u64) << 56) | offset;
            Self(value)
        }

        pub fn list_of_bytes(offset: u64) -> Self {
            let value = ((ElementTag::ByteList as u64) << 56) | offset;
            Self(value)
        }
        pub fn list_of_shorts(offset: u64) -> Self {
            let value = ((ElementTag::ShortList as u64) << 56) | offset;
            Self(value)
        }
        pub fn list_of_ints(offset: u64) -> Self {
            let value = ((ElementTag::IntList as u64) << 56) | offset;
            Self(value)
        }

        pub fn list_of_longs(offset: u64) -> Self {
            let value = ((ElementTag::LongList as u64) << 56) | offset;
            Self(value)
        }

        pub fn list_of_floats(offset: u64) -> Self {
            let value = ((ElementTag::FloatList as u64) << 56) | offset;
            Self(value)
        }

        pub fn list_of_doubles(offset: u64) -> Self {
            let value = ((ElementTag::DoubleList as u64) << 56) | offset;
            Self(value)
        }

        pub fn list_of_byte_arrays(index: u32) -> Self {
            let value = ((ElementTag::ByteArrayList as u64) << 56) | index as u64;
            Self(value)
        }

        pub fn list_of_strings(index: u32) -> Self {
            let value = ((ElementTag::StringList as u64) << 56) | index as u64;
            Self(value)
        }

        pub fn list_of_lists(length: u32) -> Self {
            let approx_len = length.min(0xFF_FFFF);
            let value = ((ElementTag::ListList as u64) << 56) | (approx_len as u64) << 32;
            Self(value)
        }

        pub fn list_of_compounds(length: u32) -> Self {
            let approx_len = length.min(0xFF_FFFF);
            let value = ((ElementTag::CompoundList as u64) << 56) | (approx_len as u64) << 32;
            Self(value)
        }

        pub fn list_of_int_arrays(index: u32) -> Self {
            let value = ((ElementTag::IntArrayList as u64) << 56) | index as u64;
            Self(value)
        }

        pub fn list_of_long_arrays(index: u32) -> Self {
            let value = ((ElementTag::LongArrayList as u64) << 56) | index as u64;
            Self(value)
        }
        pub fn set_offset(&mut self, offset: u32) {
            self.0 |= offset as u64;
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct RawList<T> {
        data: &'static [u8],
        _marker: PhantomData<T>,
    }

    enum InnerElement {
        Length(u32),
        ByteArray(u64),
        String(u64),
        IntArray(u64),
        LongArray(u64),
    }
}

pub mod nbt_string {
    use std::borrow::Cow;

    use tracing::error;

    use crate::owned::nbt_string::NBTString;

    #[derive(PartialEq, Eq)]
    pub struct NBTStr {
        data: [u8],
    }

    impl NBTStr {
        #[expect(clippy::should_implement_trait)]
        pub fn from_str(str: &str) -> Cow<'_, NBTStr> {
            match simd_cesu8::encode(str) {
                Cow::Borrowed(slice) => Cow::Borrowed(NBTStr::from_slice(slice)),
                Cow::Owned(vec) => Cow::Owned(NBTString::new_from_vec(vec)),
            }
        }

        #[expect(unsafe_code)]
        pub(crate) fn from_slice(as_slice: &[u8]) -> &Self {
            //SAFETY: same layout as a u8 slice
            unsafe { std::mem::transmute(as_slice) }
        }
        pub fn to_str(&self) -> Cow<'_, str> {
            match simd_cesu8::decode(&self.data) {
                Ok(str) => str,
                Err(err) => {
                    error!("{err}");
                    Cow::Borrowed("")
                }
            }
        }
        pub fn to_str_lossy(&self) -> Cow<'_, str> {
            simd_cesu8::decode_lossy(&self.data)
        }
    }

    impl ToOwned for NBTStr {
        type Owned = NBTString;

        #[inline]
        fn to_owned(&self) -> Self::Owned {
            NBTString::from_byte_slice(&self.data)
        }

        #[inline]
        fn clone_into(&self, target: &mut Self::Owned) {
            let NBTString { data } = target;
            data.clear();
            data.extend_from_slice(&self.data);
        }
    }
}

#[cfg(test)]
mod borrow_test {
    use crate::nbt_ids::*;

    use super::nbt_compound::RootNBTCompound;

    #[test]
    pub fn ptr_metadata() {
        let one: [u8; 4] = 1u32.to_be_bytes();
        let data: Vec<u8> = vec![
            COMPOUND_ID,
            LONG_ID,
            1u8,
            0u8,
            0u8,
            0u8,
            b'L',
            0u8,
            0u8,
            0u8,
            0u8,
            0u8,
            0u8,
            0u8,
            0u8,
            END_ID,
        ];
        let compound = RootNBTCompound::from_file(&data).expect("NBT parse error");
    }
}

pub mod list {

    pub struct ParsingError {
        kind: ParsingErrorKind,
    }
    impl ParsingError {
        pub const INTERNAL: Self = ParsingError {
            kind: ParsingErrorKind::InternalError,
        };
        const MAX_DEPTH_EXCEEDED: Self = ParsingError {
            kind: ParsingErrorKind::MaxDepthExceeded,
        };
    }
    pub enum ParsingErrorKind {
        UnexpectedEOF,
        MaxDepthExceeded,
        InternalError,
    }

    const MAX_PARSE_DEPTH: usize = 512;
    #[derive(Debug, Clone, Copy)]
    pub enum CringeCompoundTypes {
        Compound,
        ListOfCompounds,
        ListOfLists,
    }

    #[derive(Debug)]
    pub struct ParseStackEntry {
        ty: CringeCompoundTypes,
        index: u32,
    }
    impl ParseStackEntry {
        pub(crate) fn ty(&self) -> CringeCompoundTypes {
            self.ty
        }
        pub(crate) fn compound(index: u32) -> ParseStackEntry {
            ParseStackEntry {
                ty: CringeCompoundTypes::Compound,
                index,
            }
        }
        pub(crate) fn list_of_compounds(element_index: u32) -> Self {
            Self {
                ty: CringeCompoundTypes::ListOfCompounds,
                index: element_index,
            }
        }
        pub(crate) fn list_of_lists(element_index: u32) -> Self {
            Self {
                ty: CringeCompoundTypes::ListOfLists,
                index: element_index,
            }
        }
        pub(crate) fn index(&self) -> usize {
            self.index as usize
        }
    }

    #[derive(Debug)]
    pub struct ParsingStack {
        stack: [Option<ParseStackEntry>; MAX_PARSE_DEPTH],
        remaining_list_elements: [u32; MAX_PARSE_DEPTH],
        depth: usize,
    }
    impl Default for ParsingStack {
        fn default() -> Self {
            Self {
                stack: [const { None }; MAX_PARSE_DEPTH],
                remaining_list_elements: [0; MAX_PARSE_DEPTH],
                depth: Default::default(),
            }
        }
    }

    impl ParsingStack {
        pub fn push(&mut self, entry: ParseStackEntry) -> Result<(), ParsingError> {
            match self.stack.get_mut(self.depth) {
                Some(stack_top) => {
                    let _ = stack_top.insert(entry);
                }
                None => {
                    return Err(ParsingError::INTERNAL);
                }
            }
            self.depth += 1;
            if self.depth >= MAX_PARSE_DEPTH {
                return Err(ParsingError::MAX_DEPTH_EXCEEDED);
            }
            Ok(())
        }
        pub fn pop(&mut self) -> Result<ParseStackEntry, ParsingError> {
            self.depth -= 1;
            match self.stack.get_mut(self.depth) {
                Some(entry) => match entry.take() {
                    Some(entry) => Ok(entry),
                    None => Err(ParsingError::INTERNAL),
                },
                None => Err(ParsingError::INTERNAL),
            }
        }

        pub fn set_list_length(&mut self, length: u32) -> Result<(), ParsingError> {
            match self.remaining_list_elements.get_mut(self.depth - 1) {
                Some(value) => {
                    *value = length;
                    Ok(())
                }
                None => Err(ParsingError::INTERNAL),
            }
        }

        pub fn decrement_list_length(&mut self) -> Result<(), ParsingError> {
            match self.remaining_list_elements.get_mut(self.depth - 1) {
                Some(value) => {
                    *value -= 1;
                    Ok(())
                }
                None => Err(ParsingError::INTERNAL),
            }
        }

        pub fn remaining_elements_in_list(&self) -> Result<u32, ParsingError> {
            match self.remaining_list_elements.get(self.depth - 1) {
                Some(value) => Ok(*value),
                None => Err(ParsingError::INTERNAL),
            }
        }

        pub fn peek(&self) -> Result<&ParseStackEntry, ParsingError> {
            match self.stack.get(self.depth - 1) {
                Some(entry) => match entry.as_ref() {
                    Some(entry) => Ok(entry),
                    None => Err(ParsingError::INTERNAL),
                },
                None => Err(ParsingError::INTERNAL),
            }
        }

        pub fn peek_mut(&mut self) -> Result<&mut ParseStackEntry, ParsingError> {
            match self.stack.get_mut(self.depth - 1) {
                Some(entry) => match entry.as_mut() {
                    Some(entry) => Ok(entry),
                    None => Err(ParsingError::INTERNAL),
                },
                None => Err(ParsingError::INTERNAL),
            }
        }

        pub(crate) fn is_empty(&self) -> bool {
            self.depth == 0
        }
    }
}
