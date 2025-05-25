use std::borrow::Cow;
use std::str::from_utf8;

use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;
use cesu8::from_java_cesu8;
use cesu8::is_valid_java_cesu8;
use cesu8::to_java_cesu8;

use super::nbt_error::NBTError;

//wrapper for a string that came from java that needs some conversion before being de/serialized
#[derive(Debug, Clone)]
pub struct JavaString<'a> {
    pub data: &'a [u8],
}

impl<'a> JavaString<'a> {
    pub fn to_str(&self) -> Cow<'_, str> {
        from_java_cesu8(&self.data).unwrap()
    }
    pub fn new(bytes: &[u8]) -> Result<JavaString, NBTError> {
        let str = str::from_utf8(bytes);
        match str {
            Ok(_) => Ok(JavaString { data: bytes }),
            Err(err) => Err(err.into()),
        }
    }
}
