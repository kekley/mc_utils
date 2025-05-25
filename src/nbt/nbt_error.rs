use std::str::Utf8Error;

use num_enum::{TryFromPrimitive, TryFromPrimitiveError};

pub struct NBTError {
    pub kind: NBTErrorKind,
}

pub enum NBTErrorKind {
    InvalidTag(String),
    InvalidString(String),
    ListError(String),
    InvalidUTF8(String),
}

impl<T: TryFromPrimitive> From<TryFromPrimitiveError<T>> for NBTError {
    fn from(value: TryFromPrimitiveError<T>) -> Self {
        Self {
            kind: NBTErrorKind::InvalidTag(format!(
                "{:?} is not a valid NBT tag value",
                value.number
            )),
        }
    }
}

impl From<cesu8::Cesu8DecodingError> for NBTError {
    fn from(value: cesu8::Cesu8DecodingError) -> Self {
        NBTError {
            kind: NBTErrorKind::InvalidTag(value.to_string()),
        }
    }
}
impl Into<NBTError> for Utf8Error {
    fn into(self) -> NBTError {
        NBTError {
            kind: NBTErrorKind::InvalidUTF8(self.to_string()),
        }
    }
}
