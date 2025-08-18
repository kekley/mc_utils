use byteorder::{NativeEndian, ReadBytesExt};

use super::BlockStateTrait;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockState {
    data: Vec<u8>,
}

impl BlockState {
    ///Fails if `block_name.len()` does not fit in a u16
    pub fn new(block_name: &str) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(block_name.len() + size_of::<u16>());

        Self::append_str_to_vec(&mut data, block_name);
        Self { data }
    }

    fn append_str_to_vec(vec: &mut Vec<u8>, string: &str) {
        let string_length: u16 = string
            .len()
            .try_into()
            .expect("Blockstate strings should be less than 65535 bytes");

        let length_bytes = string_length.to_ne_bytes();

        vec.extend(&length_bytes);

        vec.extend(string.as_bytes());
    }

    pub fn add_property(&mut self, property_name: &str, property_value: &str) {
        Self::append_str_to_vec(&mut self.data, property_name);

        Self::append_str_to_vec(&mut self.data, property_value);
    }

    pub fn get_block_name(&self) -> &str {
        todo!()
    }

    pub fn iter_properties(&self) -> PropertiesIterator<'_> {
        todo!();
    }
}

pub struct PropertiesIterator<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Iterator for PropertiesIterator<'a> {
    type Item = (&'a str, &'a str);

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
            let property_name = str::from_utf8(str_slice).ok()?;

            self.offset += str_len as usize;

            let str_len = self.data.read_u16::<NativeEndian>().ok()?;

            self.offset += size_of_short;

            let start = self.offset;
            let end = self.offset + str_len as usize;

            if let Some(str_slice) = self.data.get(start..end) {
                self.offset += str_len as usize;

                let property_value = str::from_utf8(str_slice).ok()?;

                Some((property_name, property_value))
            } else {
                panic!("Internal error");
            }
        } else {
            panic!("Internal error");
        }
    }
}

impl<'a> BlockStateTrait<'a> for &'a BlockState {
    type StringType = &'a str;

    fn name(&self) -> Self::StringType {
        todo!()
    }

    fn iter_properties(&self) -> PropertiesIterator<'a> {
        todo!()
    }
}
