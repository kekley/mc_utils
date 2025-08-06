use crate::resource_error::ResourceErrorKind;

use super::BlockStateTrait;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockState {
    block_name: String,
    properties: Vec<(String, String)>,
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

    type PropertiesIter = std::slice::Iter<'a, (String, String)>;

    fn name(&self) -> Self::StringType {
        self.block_name.as_str()
    }

    fn iter_properties(&self) -> Self::PropertiesIter {
        self.properties.as_slice().iter()
    }
}
