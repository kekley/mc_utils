#[allow(clippy::todo)]
pub(crate) mod nbt_tag {
    use core::slice;

    use byteorder::{BigEndian, ByteOrder};

    use crate::borrow::nbt_compound::unaligned_types;

    use super::{
        nbt_compound::{Compound, Element, ElementTag, InnerElement},
        nbt_list::ListType,
        nbt_string::NBTStr,
    };

    #[derive(Debug)]
    pub struct NBTTag<'a, 'root> {
        data: &'a [u8],
        //this slice starts at the element of this tag
        elements: &'root [Element],
        inner_elements: &'root [InnerElement],
    }
    #[expect(unsafe_code)]
    impl<'a, 'root> NBTTag<'a, 'root> {
        pub fn get_byte(&self) -> Option<i8> {
            if self.elements[0].get_id() == ElementTag::Byte {
                Some(self.elements[0].get_byte())
            } else {
                None
            }
        }
        pub fn get_short(&self) -> Option<i16> {
            if self.elements[0].get_id() == ElementTag::Short {
                Some(self.elements[0].get_short())
            } else {
                None
            }
        }
        pub fn get_int(&self) -> Option<i32> {
            if self.elements[0].get_id() == ElementTag::Int {
                Some(self.elements[0].get_int())
            } else {
                None
            }
        }
        pub fn get_long(&self) -> Option<i64> {
            if self.elements[0].get_id() == ElementTag::Long {
                let offset = self.elements[0].get_long_offset() as usize;
                let size_of_long = 8_usize;
                let value = BigEndian::read_i64(&self.data[offset..offset + size_of_long]);
                Some(value)
            } else {
                None
            }
        }
        pub fn get_float(&self) -> Option<f32> {
            if self.elements[0].get_id() == ElementTag::Float {
                Some(self.elements[0].get_float())
            } else {
                None
            }
        }
        pub fn get_double(&self) -> Option<f64> {
            if self.elements[0].get_id() == ElementTag::Double {
                let offset = self.elements[0].get_double_offset() as usize;
                let size_of_double = 8_usize;
                let value = BigEndian::read_f64(&self.data[offset..offset + size_of_double]);
                Some(value)
            } else {
                None
            }
        }
        pub fn get_string(&self) -> Option<&NBTStr> {
            if self.elements[0].get_id() == ElementTag::String {
                let length_offset = self.elements[0].get_string_offset() as usize;
                let size_of_short = 2_usize;
                let data_offset = length_offset + size_of_short;
                let length_of_string =
                    BigEndian::read_u16(&self.data[length_offset..length_offset + size_of_short]);
                let byte_slice = &self.data[data_offset..data_offset + length_of_string as usize];
                Some(NBTStr::from_slice(byte_slice))
            } else {
                None
            }
        }

        pub fn get_byte_array(&self) -> Option<&[i8]> {
            if self.elements[0].get_id() == ElementTag::ByteArray {
                let length_offset = self.elements[0].get_byte_array_offset() as usize;

                let size_of_int = 4_usize;
                let data_offset = length_offset + size_of_int;

                let length_of_array =
                    BigEndian::read_u32(&self.data[length_offset..length_offset + size_of_int]);

                let ptr = &self.data[data_offset..data_offset + length_of_array as usize].as_ptr();

                let byte_slice: &[i8] =
                    unsafe { slice::from_raw_parts(ptr.cast(), length_of_array as usize) };

                Some(byte_slice)
            } else {
                None
            }
        }

        pub fn get_int_array(&self) -> Option<&[unaligned_types::BigEndianInt]> {
            if self.elements[0].get_id() == ElementTag::IntArray {
                let length_offset = self.elements[0].get_int_array_offset() as usize;

                let size_of_int = 4_usize;

                let data_offset = length_offset + size_of_int;

                let length_of_array =
                    BigEndian::read_u32(&self.data[length_offset..length_offset + size_of_int]);
                let length_of_array_in_bytes = length_of_array as usize * size_of_int;

                let ptr = &self.data[data_offset..data_offset + length_of_array_in_bytes].as_ptr();

                let slice: &[unaligned_types::BigEndianInt] =
                    unsafe { slice::from_raw_parts(ptr.cast(), length_of_array as usize) };

                Some(slice)
            } else {
                None
            }
        }

        pub fn get_long_array(&self) -> Option<&[unaligned_types::BigEndianLong]> {
            if self.elements[0].get_id() == ElementTag::LongArray {
                let length_offset = self.elements[0].get_long_array_offset() as usize;

                let size_of_int = 4_usize;

                let data_offset = length_offset + size_of_int;

                let length_of_array =
                    BigEndian::read_u32(&self.data[length_offset..length_offset + size_of_int]);

                let size_of_long = 4_usize;
                let length_of_array_in_bytes = length_of_array as usize * size_of_long;

                let ptr = self.data[data_offset..data_offset + length_of_array_in_bytes].as_ptr();

                let slice: &[unaligned_types::BigEndianLong] =
                    unsafe { slice::from_raw_parts(ptr.cast(), length_of_array as usize) };

                Some(slice)
            } else {
                None
            }
        }

        pub fn get_list(&self) -> Option<ListType<'a, 'root>> {
            //we want to panic if the slice is empty
            if self.elements[0].get_id().is_list() {
                Some(ListType::new(self.data, self.elements, self.inner_elements))
            } else {
                None
            }
        }
        pub fn get_compound(&self) -> Option<Compound<'a, 'root>> {
            if self.elements[0].get_id() == ElementTag::Compound {
                Some(Compound::new(self.data, self.elements, self.inner_elements))
            } else {
                None
            }
        }

        pub(crate) fn new(
            data: &'a [u8],
            element_slice: &'root [Element],
            inner_elements: &'root [InnerElement],
        ) -> Self {
            Self {
                data,
                elements: element_slice,
                inner_elements,
            }
        }
    }
}

pub mod nbt_compound {

    use crate::borrow::nbt_tag::NBTTag;
    use std::fmt::Debug;
    use std::io::Cursor;

    use crate::borrow::parsing_stack::ParseStackEntry;
    use crate::borrow::parsing_stack::ParsingStack;
    use crate::{nbt_error::NBTError, nbt_ids::*};
    use byteorder::{BigEndian, ReadBytesExt};
    use bytes::Buf;
    use num_enum::TryFromPrimitive;

    use super::nbt_string::NBTStr;
    use super::parsing_stack::ParsingError;

    pub struct Compound<'a, 'root> {
        data: &'a [u8],
        elements: &'root [Element],
        inner_elements: &'root [InnerElement],
    }

    impl<'a, 'root> Compound<'a, 'root> {
        pub fn new(
            data: &'a [u8],
            elements: &'root [Element],
            inner_elements: &'root [InnerElement],
        ) -> Self {
            let compound_element = elements[0];

            let max_offset = compound_element.get_offset() as usize;
            let compound_elements = &elements[..max_offset];

            Self {
                data,
                elements: compound_elements,
                inner_elements,
            }
        }

        pub fn get_tag(&self, name: &str) -> Option<NBTTag<'a, 'root>> {
            let name = NBTStr::from_str(name);
            let name = name.as_ref();
            let tag = self.iter().find(|(tag_name, _tag)| *tag_name == name);

            Some(tag?.1)
        }
        pub fn iter(&self) -> NBTCompoundIter<'a, 'root> {
            let max_tape_offset = self.elements[0].get_truncated_len_and_offset().1 as usize;

            let element_slice = &self.elements[1..max_tape_offset];

            NBTCompoundIter {
                current_offset: 0,
                data: self.data,
                elements: element_slice,
                inner_elements: self.inner_elements,
            }
        }
    }

    pub struct NBTCompoundIter<'a, 'root> {
        current_offset: usize,
        data: &'a [u8],
        elements: &'root [Element],
        inner_elements: &'root [InnerElement],
    }
    impl<'a, 'root> Iterator for NBTCompoundIter<'a, 'root> {
        type Item = (&'a NBTStr, NBTTag<'a, 'root>);

        fn next(&mut self) -> Option<Self::Item> {
            if self.current_offset + 1 >= self.elements.len() {
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

            let nbt_str = NBTStr::from_slice(string_slice);

            self.current_offset += 1;

            let element_slice = &self.elements[self.current_offset..];

            self.current_offset += element_slice[0].get_skip_amount();
            Some((
                nbt_str,
                NBTTag::new(self.data, element_slice, self.inner_elements),
            ))
        }
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
            let nbt_str = NBTStr::from_slice(slice);
            assert!(cursor.remaining() >= len as usize);
            cursor.advance(len as usize);
            Ok(nbt_str)
        }
        pub fn name(&self) -> &NBTStr {
            self.name
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
                    crate::borrow::parsing_stack::CringeCompoundTypes::Compound => {
                        Self::read_tag_in_compound(
                            &mut cursor,
                            &mut elements,
                            &mut inner_elements,
                            &mut parsing_stack,
                        )?;
                    }
                    crate::borrow::parsing_stack::CringeCompoundTypes::ListOfCompounds => {
                        Self::read_compound_in_list(
                            &mut cursor,
                            &mut elements,
                            &mut inner_elements,
                            &mut parsing_stack,
                        )?;
                    }
                    crate::borrow::parsing_stack::CringeCompoundTypes::ListOfLists => {
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
        pub fn get_tag<'root>(&'root self, name: &str) -> Option<NBTTag<'a, 'root>> {
            let name = NBTStr::from_str(name);
            let name = name.as_ref();
            let tag = self.iter().find(|(tag_name, _)| *tag_name == name);

            Some(tag?.1)
        }

        pub fn iter<'root>(&'root self) -> NBTCompoundIter<'a, 'root> {
            let max_tape_offset = self.elements[0].get_truncated_len_and_offset().1 as usize;

            let element_slice = &self.elements[1..max_tape_offset];

            NBTCompoundIter {
                current_offset: 0,
                data: self.data,
                elements: element_slice,
                inner_elements: &self.inner_elements,
            }
        }
        fn read_tag_in_compound(
            cursor: &mut Cursor<&[u8]>,
            elements: &mut Vec<Element>,
            inner_elements: &mut Vec<InnerElement>,
            stack: &mut ParsingStack,
        ) -> Result<(), NBTError> {
            let tag_id = Self::read_tag_id(cursor)?;

            let element = match tag_id {
                NBTId::EndId => {
                    Self::handle_compound_end(elements, stack)?;
                    return Ok(());
                }
                NBTId::ByteId => {
                    let name_element = Element::name_string(cursor.position());
                    Self::read_u16_length_and_slice_of(cursor, 1)?;
                    elements.push(name_element);

                    let value = cursor.read_u8()?;
                    Element::from_byte(value)
                }
                NBTId::ShortId => {
                    let name_element = Element::name_string(cursor.position());
                    Self::read_u16_length_and_slice_of(cursor, 1)?;
                    elements.push(name_element);

                    let value = cursor.read_u16::<BigEndian>()?;
                    Element::from_short(value)
                }
                NBTId::IntId => {
                    let name_element = Element::name_string(cursor.position());
                    Self::read_u16_length_and_slice_of(cursor, 1)?;
                    elements.push(name_element);

                    let value = cursor.read_u32::<BigEndian>()?;
                    Element::from_int(value)
                }
                NBTId::LongId => {
                    let name_element = Element::name_string(cursor.position());
                    Self::read_u16_length_and_slice_of(cursor, 1)?;
                    elements.push(name_element);

                    let offset = cursor.position();
                    //we need to advance the cursor since we just stored an offset
                    cursor.advance(8);
                    Element::from_long(offset)
                }
                NBTId::FloatId => {
                    let name_element = Element::name_string(cursor.position());
                    Self::read_u16_length_and_slice_of(cursor, 1)?;
                    elements.push(name_element);

                    let value = cursor.read_u32::<BigEndian>()?;
                    Element::from_float(value)
                }
                NBTId::DoubleId => {
                    let name_element = Element::name_string(cursor.position());
                    Self::read_u16_length_and_slice_of(cursor, 1)?;
                    elements.push(name_element);

                    let offset = cursor.position();

                    //we need to advance the cursor since we just stored an offset
                    cursor.advance(8);
                    Element::from_double(offset)
                }
                NBTId::ByteArrayId => {
                    let name_element = Element::name_string(cursor.position());
                    Self::read_u16_length_and_slice_of(cursor, 1)?;
                    elements.push(name_element);

                    let offset = cursor.position();
                    Self::read_u32_length_and_slice_of(cursor, 1)?;
                    Element::from_byte_array(offset)
                }
                NBTId::StringId => {
                    let name_element = Element::name_string(cursor.position());
                    Self::read_u16_length_and_slice_of(cursor, 1)?;
                    elements.push(name_element);

                    let offset = cursor.position();
                    Self::read_u16_length_and_slice_of(cursor, 1)?;
                    Element::from_string(offset)
                }
                NBTId::LongArrayId => {
                    let name_element = Element::name_string(cursor.position());
                    Self::read_u16_length_and_slice_of(cursor, 1)?;
                    elements.push(name_element);

                    let offset = cursor.position();
                    Self::read_u32_length_and_slice_of(cursor, 8)?;
                    Element::from_long_array(offset)
                }
                NBTId::IntArrayId => {
                    let name_element = Element::name_string(cursor.position());
                    Self::read_u16_length_and_slice_of(cursor, 1)?;
                    elements.push(name_element);

                    let offset = cursor.position();
                    Self::read_u32_length_and_slice_of(cursor, 4)?;
                    Element::from_int_array(offset)
                }
                NBTId::ListId => {
                    let name_element = Element::name_string(cursor.position());
                    Self::read_u16_length_and_slice_of(cursor, 1)?;
                    elements.push(name_element);

                    return Self::list(cursor, elements, inner_elements, stack);
                }
                NBTId::CompoundId => {
                    let name_element = Element::name_string(cursor.position());
                    Self::read_u16_length_and_slice_of(cursor, 1)?;
                    elements.push(name_element);

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
                    inner_elements.push(InnerElement::Length(length.into()));
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
                    inner_elements.push(InnerElement::Length(length.into()));
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
                    inner_elements.push(InnerElement::Length(length as u64));
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
                    inner_elements.push(InnerElement::Length(length as u64));
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
            _cursor: &mut Cursor<&[u8]>,
            elements: &mut Vec<Element>,
            _inner_elements: &mut [InnerElement],
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
            inner_elements: &mut [InnerElement],
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
            elements: &mut [Element],
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
        pub struct BigEndianLong(pub u64);

        #[derive(Debug, Clone, Copy)]
        #[repr(C, packed)]
        pub struct BigEndianDouble(pub u64);

        #[derive(Debug, Clone, Copy)]
        #[repr(C, packed)]
        pub struct BigEndianInt(pub u32);

        #[derive(Debug, Clone, Copy)]
        #[repr(C, packed)]
        pub struct BigEndianShort(pub u16);

        #[derive(Debug, Clone, Copy)]
        #[repr(C, packed)]
        pub struct BigEndianFloat(pub f32);
    }

    ///bit pattern for lists: u8 | u24 | u32
    ///           kind | len | offset
    #[derive(Clone, Copy)]
    pub struct Element(u64);

    impl Debug for Element {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_tuple("Element").field(&self.get_id()).finish()
        }
    }

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

    impl ElementTag {
        pub fn is_list(self) -> bool {
            matches!(
                self,
                ElementTag::ListList
                    | ElementTag::CompoundList
                    | ElementTag::EmptyList
                    | ElementTag::ByteList
                    | ElementTag::ShortList
                    | ElementTag::IntList
                    | ElementTag::LongList
                    | ElementTag::FloatList
                    | ElementTag::DoubleList
                    | ElementTag::ByteArrayList
                    | ElementTag::StringList
                    | ElementTag::IntArrayList
                    | ElementTag::LongArrayList
            )
        }
    }
    impl Element {
        pub fn get_byte(self) -> i8 {
            assert!(self.get_id() == ElementTag::Byte);
            (self.0 & 0xFF) as i8
        }
        pub fn get_short(self) -> i16 {
            assert!(self.get_id() == ElementTag::Short);
            (self.0 & 0xFFFF) as i16
        }
        pub fn get_int(self) -> i32 {
            assert!(self.get_id() == ElementTag::Int);
            (self.0 & 0xFFFF_FFFF) as i32
        }
        pub fn get_long_offset(self) -> u64 {
            assert!(self.get_id() == ElementTag::Long);
            self.0 & 0x00FF_FFFF_FFFF_FFFF
        }
        pub fn get_float(self) -> f32 {
            assert!(self.get_id() == ElementTag::Float);
            (self.0 & 0xFFFF) as f32
        }
        pub fn get_double_offset(self) -> u64 {
            assert!(self.get_id() == ElementTag::Double);
            self.0 & 0x00FF_FFFF_FFFF_FFFF
        }
        pub fn get_byte_array_offset(self) -> u64 {
            assert!(self.get_id() == ElementTag::ByteArray);
            self.0 & 0x00FF_FFFF_FFFF_FFFF
        }
        pub fn get_string_offset(self) -> u64 {
            assert!(self.get_id() == ElementTag::ByteArray);
            self.0 & 0x00FF_FFFF_FFFF_FFFF
        }
        pub fn get_long_array_offset(self) -> u64 {
            assert!(self.get_id() == ElementTag::LongArray);
            self.0 & 0x00FF_FFFF_FFFF_FFFF
        }
        pub fn get_int_array_offset(self) -> u64 {
            assert!(self.get_id() == ElementTag::IntArray);
            self.0 & 0x00FF_FFFF_FFFF_FFFF
        }
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
                ElementTag::Compound | ElementTag::ListList | ElementTag::CompoundList => {
                    self.get_truncated_len_and_offset().1 as usize
                }
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

    #[derive(Debug, Clone)]
    pub enum InnerElement {
        Length(u64),
        ByteArray(u64),
        String(u64),
        IntArray(u64),
        LongArray(u64),
    }
}
#[allow(dead_code)]
#[allow(clippy::todo)]
pub mod nbt_list {
    use core::slice;
    use std::marker::PhantomData;

    use byteorder::{BigEndian, ByteOrder};

    use crate::borrow::nbt_compound::ElementTag;

    use super::{
        nbt_compound::{
            unaligned_types::{
                BigEndianDouble, BigEndianFloat, BigEndianInt, BigEndianLong, BigEndianShort,
            },
            Compound, Element, InnerElement,
        },
        nbt_string::NBTStr,
    };
    pub enum ListType<'a, 'root> {
        Empty(()),
        Byte(PrimitiveList<'a, i8>),
        Short(PrimitiveList<'a, BigEndianShort>),
        Int(PrimitiveList<'a, BigEndianInt>),
        Long(PrimitiveList<'a, BigEndianLong>),
        Float(PrimitiveList<'a, BigEndianFloat>),
        Double(PrimitiveList<'a, BigEndianDouble>),
        ByteArray(ArrayList<'a, 'root, i8>),
        IntArray(ArrayList<'a, 'root, BigEndianInt>),
        LongArray(ArrayList<'a, 'root, BigEndianLong>),
        String(StringList<'a, 'root>),
        List(ListList<'a, 'root>),
        Compound(CompoundList<'a, 'root>),
    }
    impl<'a, 'root> ListType<'a, 'root> {
        pub fn new(
            data: &'a [u8],
            elements: &'root [Element],
            inner_elements: &'root [InnerElement],
        ) -> Self {
            let list_element = elements[0];
            match list_element.get_id() {
                ElementTag::EmptyList => ListType::empty_list(list_element),
                ElementTag::ByteList => ListType::byte_list(list_element, data),
                ElementTag::ShortList => ListType::short_list(list_element, data),
                ElementTag::IntList => ListType::int_list(list_element, data),
                ElementTag::LongList => ListType::long_list(list_element, data),
                ElementTag::FloatList => ListType::float_list(list_element, data),
                ElementTag::DoubleList => ListType::double_list(list_element, data),
                ElementTag::ByteArrayList => {
                    ListType::byte_array_list(data, list_element, inner_elements)
                }
                ElementTag::StringList => ListType::string_list(data, list_element, inner_elements),
                ElementTag::ListList => ListType::list_list(data, elements, inner_elements),
                ElementTag::CompoundList => ListType::compound_list(data, elements, inner_elements),
                ElementTag::IntArrayList => {
                    ListType::int_array_list(data, list_element, inner_elements)
                }
                ElementTag::LongArrayList => {
                    ListType::long_array_list(data, list_element, inner_elements)
                }
                _ => {
                    unreachable!()
                }
            }
        }
        pub fn empty_list(list_element: Element) -> ListType<'a, 'root> {
            assert!(list_element.get_id() == ElementTag::EmptyList);
            ListType::Empty(())
        }

        pub fn byte_list(list_element: Element, data: &'a [u8]) -> ListType<'a, 'root> {
            assert!(list_element.get_id() == ElementTag::ByteList);
            ListType::Byte(PrimitiveList::new(data, list_element))
        }

        pub fn short_list(list_element: Element, data: &'a [u8]) -> ListType<'a, 'root> {
            assert!(list_element.get_id() == ElementTag::ShortList);
            ListType::Short(PrimitiveList::new(data, list_element))
        }

        pub fn int_list(list_element: Element, data: &'a [u8]) -> ListType<'a, 'root> {
            assert!(list_element.get_id() == ElementTag::IntList);

            ListType::Int(PrimitiveList::new(data, list_element))
        }
        pub fn long_list(list_element: Element, data: &'a [u8]) -> ListType<'a, 'root> {
            assert!(list_element.get_id() == ElementTag::LongList);
            ListType::Long(PrimitiveList::new(data, list_element))
        }
        pub fn float_list(list_element: Element, data: &'a [u8]) -> ListType<'a, 'root> {
            assert!(list_element.get_id() == ElementTag::FloatList);
            ListType::Float(PrimitiveList::new(data, list_element))
        }
        pub fn double_list(list_element: Element, data: &'a [u8]) -> ListType<'a, 'root> {
            assert!(list_element.get_id() == ElementTag::DoubleList);
            ListType::Double(PrimitiveList::new(data, list_element))
        }
        pub fn byte_array_list(
            data: &'a [u8],
            list_element: Element,
            inner_elements: &'root [InnerElement],
        ) -> ListType<'a, 'root> {
            assert!(list_element.get_id() == ElementTag::ByteArrayList);
            let inner_index = list_element.get_offset() as usize;
            let array_elements = if let InnerElement::Length(length) = inner_elements[inner_index] {
                &inner_elements[inner_index..inner_index + length as usize]
            } else {
                unreachable!()
            };
            ListType::ByteArray(ArrayList {
                data,
                inner_elements: array_elements,
                phantom_data: PhantomData,
            })
        }
        pub fn int_array_list(
            data: &'a [u8],
            list_element: Element,
            inner_elements: &'root [InnerElement],
        ) -> ListType<'a, 'root> {
            assert!(list_element.get_id() == ElementTag::IntArrayList);
            let inner_index = list_element.get_offset() as usize;
            let array_elements = if let InnerElement::Length(length) = inner_elements[inner_index] {
                &inner_elements[inner_index..inner_index + length as usize]
            } else {
                unreachable!()
            };
            ListType::IntArray(ArrayList {
                data,
                inner_elements: array_elements,
                phantom_data: PhantomData,
            })
        }
        pub fn long_array_list(
            data: &'a [u8],
            list_element: Element,
            inner_elements: &'root [InnerElement],
        ) -> ListType<'a, 'root> {
            assert!(list_element.get_id() == ElementTag::LongArrayList);
            let inner_index = list_element.get_offset() as usize;
            let array_elements = if let InnerElement::Length(length) = inner_elements[inner_index] {
                &inner_elements[inner_index..inner_index + length as usize]
            } else {
                unreachable!()
            };
            ListType::LongArray(ArrayList {
                data,
                inner_elements: array_elements,
                phantom_data: PhantomData,
            })
        }
        pub fn string_list(
            data: &'a [u8],
            list_element: Element,
            inner_elements: &'root [InnerElement],
        ) -> ListType<'a, 'root> {
            assert!(list_element.get_id() == ElementTag::StringList);
            let inner_index = list_element.get_offset() as usize;
            let array_elements = if let InnerElement::Length(length) = inner_elements[inner_index] {
                &inner_elements[inner_index..inner_index + length as usize]
            } else {
                unreachable!()
            };

            ListType::String(StringList {
                data,
                inner_elements: array_elements,
            })
        }

        pub fn list_list(
            data: &'a [u8],
            elements: &'root [Element],
            inner_elements: &'root [InnerElement],
        ) -> ListType<'a, 'root> {
            let list_element = elements
                .first()
                .expect("list list element should be first in the slice passed to list_list");

            assert!(list_element.get_id() == ElementTag::ListList);
            let (trunc_len, offset) = list_element.get_truncated_len_and_offset();
            //exclude the list list element
            let slice_of_list_elements = &elements[1..offset as usize];
            let iter = ListListIter {
                current_offset: 0,
                approx_len: trunc_len,
                data,
                elements: slice_of_list_elements,
                inner_elements,
            };

            ListType::List(ListList { iter })
        }
        pub fn compound_list(
            data: &'a [u8],
            elements: &'root [Element],
            inner_elements: &'root [InnerElement],
        ) -> ListType<'a, 'root> {
            let list_element = elements.first().expect(
                "Compound list element should be first in the slice passed to compound_list",
            );

            assert!(list_element.get_id() == ElementTag::CompoundList);
            //approximate length is not calculated for compounds
            let (_trunc_len, offset) = list_element.get_truncated_len_and_offset();
            //exclude the compound list element
            let slice_of_list_elements = &elements[1..offset as usize];

            let iter = CompoundListIter {
                current_offset: 0,
                data,
                elements: slice_of_list_elements,
                inner_elements,
            };
            ListType::Compound(CompoundList { iter })
        }
    }

    pub struct PrimitiveList<'a, T> {
        data: &'a [T],
    }

    impl<'a, T> PrimitiveList<'a, T> {
        #[expect(unsafe_code)]
        pub fn new(data: &'a [u8], list_element: Element) -> Self {
            let length_offset = list_element.get_offset() as usize;
            let size_of_int = 4_usize;
            let list_length =
                BigEndian::read_u32(&data[length_offset..length_offset + size_of_int]) as usize;
            let data_offset = length_offset + size_of_int;

            let ptr = &data[data_offset..data_offset + list_length * size_of::<T>()].as_ptr();

            let data: &[T] = unsafe { slice::from_raw_parts(ptr.cast(), list_length) };

            PrimitiveList { data }
        }
        pub fn as_slice(&self) -> &[T] {
            self.data
        }
    }

    impl<'a, T> NBTList for PrimitiveList<'a, T> {
        type Output = &'a T;

        fn get_index(&self, index: usize) -> Option<Self::Output> {
            self.data.get(index)
        }

        fn length(&self) -> usize {
            self.data.len()
        }
    }

    pub struct ArrayList<'a, 'root, T> {
        data: &'a [u8],
        inner_elements: &'root [InnerElement],
        phantom_data: PhantomData<T>,
    }

    impl<'a, 'root, T> NBTList for ArrayList<'a, 'root, T>
    where
        T: 'a,
    {
        type Output = &'a [T];
        #[expect(unsafe_code)]
        fn get_index(&self, index: usize) -> Option<Self::Output> {
            let list_element = self.inner_elements.get(index + 1)?;
            match *list_element {
                InnerElement::ByteArray(length_offset)
                | InnerElement::IntArray(length_offset)
                | InnerElement::LongArray(length_offset) => {
                    let length_offset = length_offset as usize;
                    let size_of_int = 4usize;
                    let data_offset = length_offset + size_of_int;
                    let array_length =
                        BigEndian::read_u32(&self.data[length_offset..length_offset + size_of_int])
                            as usize;
                    let ptr = self.data[data_offset..data_offset + array_length * size_of::<T>()]
                        .as_ptr();
                    let slice: &[T] = unsafe { slice::from_raw_parts(ptr.cast(), array_length) };
                    Some(slice)
                }
                _ => unreachable!(),
            }
        }

        fn length(&self) -> usize {
            let length_element = self
                .inner_elements
                .first()
                .expect("List should always have a length element");
            if let InnerElement::Length(length) = length_element {
                *length as usize
            } else {
                unreachable!()
            }
        }
    }
    pub struct StringList<'a, 'root> {
        data: &'a [u8],
        inner_elements: &'root [InnerElement],
    }

    impl<'a, 'root> NBTList for StringList<'a, 'root> {
        type Output = &'a NBTStr;

        fn get_index(&self, index: usize) -> Option<Self::Output> {
            let list_element = self.inner_elements.get(index + 1)?;

            if let InnerElement::String(length_offset) = *list_element {
                let length_offset = length_offset as usize;
                let size_of_short = 2usize;
                let data_offset = length_offset + size_of_short;
                let string_length =
                    BigEndian::read_u32(&self.data[length_offset..length_offset + size_of_short])
                        as usize;
                let slice = &self.data[data_offset..data_offset + string_length];

                Some(NBTStr::from_slice(slice))
            } else {
                unreachable!()
            }
        }

        fn length(&self) -> usize {
            let length_element = self
                .inner_elements
                .first()
                .expect("List should always have a length element");
            if let InnerElement::Length(length) = length_element {
                *length as usize
            } else {
                unreachable!()
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct ListList<'a, 'root> {
        iter: ListListIter<'a, 'root>,
    }

    impl<'root, 'a> NBTList for ListList<'a, 'root> {
        type Output = ListType<'a, 'root>;

        fn get_index(&self, index: usize) -> Option<Self::Output> {
            self.iter.clone().nth(index)
        }

        fn length(&self) -> usize {
            self.iter.length()
        }
    }
    #[derive(Debug, Clone)]
    pub struct ListListIter<'a, 'root> {
        current_offset: usize,
        approx_len: u32,
        data: &'a [u8],
        elements: &'root [Element],
        inner_elements: &'root [InnerElement],
    }
    impl<'a, 'root> ListListIter<'a, 'root> {
        pub fn approx_len(&self) -> usize {
            self.approx_len as usize
        }
        pub fn length(&self) -> usize {
            let len = self.approx_len() as u32;
            if len < (2u32.pow(24)) {
                len as usize
            } else {
                self.clone().count()
            }
        }
    }

    impl<'a, 'root> Iterator for ListListIter<'a, 'root> {
        type Item = ListType<'a, 'root>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.current_offset + 1 >= self.elements.len() {
                return None;
            }

            let current_list_element = self.elements[self.current_offset];
            assert!(current_list_element.get_id().is_list());

            let skip_offset = if matches!(
                current_list_element.get_id(),
                ElementTag::CompoundList | ElementTag::ListList
            ) {
                current_list_element.get_offset() as usize
            } else {
                1
            };
            let return_list = ListType::new(self.data, self.elements, self.inner_elements);
            self.current_offset += skip_offset;
            Some(return_list)
        }
    }

    pub struct CompoundList<'a, 'root> {
        iter: CompoundListIter<'a, 'root>,
    }

    impl<'root, 'a> NBTList for CompoundList<'a, 'root> {
        type Output = Compound<'a, 'root>;

        fn get_index(&self, index: usize) -> Option<Self::Output> {
            self.iter.clone().nth(index)
        }

        fn length(&self) -> usize {
            self.iter.clone().length()
        }
    }
    #[derive(Debug, Clone)]
    pub struct CompoundListIter<'a, 'root> {
        current_offset: usize,
        data: &'a [u8],
        elements: &'root [Element],
        inner_elements: &'root [InnerElement],
    }

    impl<'a, 'root> CompoundListIter<'a, 'root> {
        pub fn length(&self) -> usize {
            self.clone().count()
        }
    }
    impl<'a, 'root> Iterator for CompoundListIter<'a, 'root> {
        type Item = Compound<'a, 'root>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.current_offset + 1 >= self.elements.len() {
                return None;
            }

            let compound_element = self.elements[self.current_offset];
            let skip_amount = compound_element.get_offset() as usize;
            let compound = Compound::new(self.data, self.elements, self.inner_elements);

            self.current_offset += skip_amount;
            Some(compound)
        }
    }

    pub trait NBTList {
        type Output;
        fn get_index(&self, index: usize) -> Option<Self::Output>;
        fn length(&self) -> usize;
    }
}

pub mod nbt_string {
    use std::{borrow::Cow, fmt::Display};

    use tracing::error;

    use crate::owned::nbt_string::NBTString;

    #[derive(PartialEq, Eq)]
    pub struct NBTStr {
        data: [u8],
    }

    impl Display for NBTStr {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.to_str_lossy())
        }
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

pub mod parsing_stack {
    #[allow(dead_code)]
    #[derive(Debug)]
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
    #[derive(Debug)]
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

#[cfg(test)]
mod borrow_test {

    use super::nbt_compound::RootNBTCompound;

    #[test]
    pub fn compound() {
        let path = "./test_assets/iceandfire_myrmex.dat";
        let level_dat = std::fs::read(path).unwrap_or_else(|_| panic!("could not find {path}"));

        let compound = RootNBTCompound::from_file(&level_dat).expect("NBT parse error");
        let tag = compound.get_tag("data").expect("Could not get data tag");

        let data_compound = tag.get_compound().expect("Tag was not compound");

        data_compound.iter().for_each(|(name, _tag)| {
            println!("{name}");
        });
    }
}
