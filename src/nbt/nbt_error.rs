use std::{io, str::Utf8Error};

use num_enum::{TryFromPrimitive, TryFromPrimitiveError};

#[derive(Debug)]
pub struct NBTError {
    pub kind: NBTErrorKind,
}

#[derive(Debug)]
pub enum NBTErrorKind {
    InvalidNBT(String),
    InvalidTag(String),
    InvalidString(String),
    ListError(String),
    InvalidUTF8(String),
    IOError(io::Error),
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
impl From<Utf8Error> for NBTError {
    fn from(val: Utf8Error) -> Self {
        NBTError {
            kind: NBTErrorKind::InvalidUTF8(val.to_string()),
        }
    }
}

impl From<std::io::Error> for NBTError {
    fn from(value: std::io::Error) -> Self {
        Self {
            kind: NBTErrorKind::IOError(value),
        }
    }
}
