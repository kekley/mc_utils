use byteorder::{NativeEndian, ReadBytesExt};

use crate::borrow::nbt_string::NBTStr;

use super::BlockStateTrait;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockState {
    data: Vec<u8>,
}

impl BlockState {
    ///Fails if `block_name.len()` does not fit in a u16
    pub fn new(block_name: &NBTStr) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(block_name.as_bytes().len() + size_of::<u16>());

        Self::append_str_to_vec(&mut data, block_name);
        Self { data }
    }

    fn append_str_to_vec(vec: &mut Vec<u8>, string: &NBTStr) {
        let string_length: u16 = string
            .as_bytes()
            .len()
            .try_into()
            .expect("Blockstate strings should be less than 65535 bytes");

        let length_bytes = string_length.to_ne_bytes();

        vec.extend(&length_bytes);

        vec.extend(string.as_bytes());
    }

    pub fn add_property(&mut self, property_name: &NBTStr, property_value: &NBTStr) {
        Self::append_str_to_vec(&mut self.data, property_name);

        Self::append_str_to_vec(&mut self.data, property_value);
    }

    pub fn get_block_name(&self) -> &NBTStr {
        let length: [u8; 2] = self
            .data
            .get(0..size_of::<u16>())
            .expect("Blockstate data should not be empty")
            .try_into()
            .unwrap();
        let length = u16::from_ne_bytes(length) as usize;
        let name_slice: &[u8] = self
            .data
            .get(2..2 + length)
            .expect("Length should be the valid length of the string");

        //SAFETY: only valid utf8 bytes were appended to this buffer
        NBTStr::from_slice(name_slice)
    }

    pub fn properties_iter(&self) -> PropertiesIterator<'_> {
        let length: [u8; 2] = self
            .data
            .get(0..size_of::<u16>())
            .expect("Blockstate data should not be empty")
            .try_into()
            .unwrap();
        let name_length = u16::from_ne_bytes(length) as usize;

        let offset = size_of::<u16>() + name_length;

        PropertiesIterator {
            data: &self.data[offset..],
            offset: 0,
        }
    }
}

pub struct PropertiesIterator<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Iterator for PropertiesIterator<'a> {
    type Item = (&'a NBTStr, &'a NBTStr);

    fn next(&mut self) -> Option<Self::Item> {
        let size_of_short = size_of::<u16>();
        if self.offset + size_of_short >= self.data.len() {
            return None;
        }

        let str_len = self.data.read_u16::<NativeEndian>().ok()?;

        self.offset += size_of_short;

        let start = self.offset;
        let end = self.offset + str_len as usize;

        if let Some(str_slice) = self.data.get(start..end) {
            let property_name = NBTStr::from_slice(str_slice);

            self.offset += str_len as usize;

            let str_len = self.data.read_u16::<NativeEndian>().ok()?;

            self.offset += size_of_short;

            let start = self.offset;
            let end = self.offset + str_len as usize;

            if let Some(str_slice) = self.data.get(start..end) {
                self.offset += str_len as usize;

                let property_value = NBTStr::from_slice(str_slice);

                Some((property_name, property_value))
            } else {
                panic!("Internal error");
            }
        } else {
            panic!("Internal error");
        }
    }
}

impl BlockStateTrait for BlockState {
    fn name(&self) -> &NBTStr {
        self.get_block_name()
    }

    #[allow(refining_impl_trait)]
    fn iter_properties(&self) -> PropertiesIterator<'_> {
        self.properties_iter()
    }
}
