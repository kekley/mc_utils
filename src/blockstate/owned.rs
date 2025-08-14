use crate::resource_error::ResourceErrorKind;

use super::BlockStateTrait;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockState {
    data: Vec<u8>,
}

impl BlockState {
    pub fn new(block_name: &str) -> Self {
        let name_length = block_name.len();

        let mut data: Vec<u8> = Vec::with_capacity(name_length + size_of::<usize>());

        let length_bytes = name_length.to_ne_bytes();

        data.extend(&length_bytes);

        data.extend(block_name.as_bytes());

        Self { data }
    }

    pub fn add_property(&mut self, property_name: &str, property_value: &str) {}
}

pub fn from_str(properties: &str) -> Result<Vec<(String, String)>, ResourceErrorKind> {
    let properties_split = properties.split(",");
    let result = properties_split
        .into_iter()
        .map(|property| {
            let (name, value) = property
                .split_once("=")
                .ok_or(ResourceErrorKind::InvalidField(String::new()))?;
            let name_string = String::from(name);
            let value_string = String::from(value);

            Ok((name_string, value_string))
        })
        .collect::<Result<Vec<_>, ResourceErrorKind>>()?;
    Ok(result)
}

impl<'a> BlockStateTrait for &'a BlockState {
    type StringType = &'a str;

    fn name(&self) -> Self::StringType {
        todo!()
    }

    fn iter_properties(&self) -> impl Iterator<Item = (&Self::StringType, &Self::StringType)> {
        todo!()
    }
}
