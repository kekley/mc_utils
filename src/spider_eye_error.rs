use num_enum::TryFromPrimitiveError;

use crate::nbt::nbt_ids::NBTId;

#[derive(Debug)]
pub enum SpiderEyeError {
    DEFAULT,
    IO(std::io::Error),
    InvalidFile(),
    InvalidOffset(isize, isize),
    UnknownCompression(u8),
    TryFromPrimitiveError(TryFromPrimitiveError<NBTId>),
    ListError(i32),
    UTF8Error(std::string::FromUtf8Error),
    JavaStringDecodingError(simd_cesu8::DecodingError),
}

impl From<std::io::Error> for SpiderEyeError {
    fn from(value: std::io::Error) -> Self {
        SpiderEyeError::IO(value)
    }
}

impl From<num_enum::TryFromPrimitiveError<NBTId>> for SpiderEyeError {
    fn from(value: num_enum::TryFromPrimitiveError<NBTId>) -> Self {
        SpiderEyeError::TryFromPrimitiveError(value)
    }
}

impl From<std::string::FromUtf8Error> for SpiderEyeError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        SpiderEyeError::UTF8Error(value)
    }
}

impl From<simd_cesu8::DecodingError> for SpiderEyeError {
    fn from(value: simd_cesu8::DecodingError) -> Self {
        SpiderEyeError::JavaStringDecodingError(value)
    }
}

impl std::fmt::Display for SpiderEyeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpiderEyeError::DEFAULT => todo!(),
            SpiderEyeError::IO(error) => f.write_fmt(format_args!("IO Error: {error:?}")),
            SpiderEyeError::InvalidOffset(x, z) => {
                f.write_fmt(format_args!("Invalid Offset: x: {x}, z: {z}"))
            }
            SpiderEyeError::UnknownCompression(val) => {
                f.write_fmt(format_args!("Unknown Compresssion sceme: {val}"))
            }
            SpiderEyeError::TryFromPrimitiveError(try_from_primitive_error) => f.write_fmt(
                format_args!("Failed Primitive Conversion: {try_from_primitive_error:?}"),
            ),
            SpiderEyeError::UTF8Error(utf_8_error) => f.write_fmt(format_args!(
                "Error creating string from bytes: {utf_8_error:?}"
            )),
            SpiderEyeError::JavaStringDecodingError(cesu8_decoding_error) => f.write_fmt(
                format_args!("Error parsing a java string: {cesu8_decoding_error:?}"),
            ),
            SpiderEyeError::ListError(len) => {
                f.write_fmt(format_args!("Error parsing list of len {len}"))
            }
            SpiderEyeError::InvalidFile() => f.write_fmt(format_args!("invalid file maybeh")),
        }
    }
}

impl std::error::Error for SpiderEyeError {}
