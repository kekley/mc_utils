use num_enum::TryFromPrimitiveError;

use crate::nbt::nbt_ids::NBTId;

#[derive(Debug)]
pub enum MCUtilsError {
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

impl From<std::io::Error> for MCUtilsError {
    fn from(value: std::io::Error) -> Self {
        MCUtilsError::IO(value)
    }
}

impl From<num_enum::TryFromPrimitiveError<NBTId>> for MCUtilsError {
    fn from(value: num_enum::TryFromPrimitiveError<NBTId>) -> Self {
        MCUtilsError::TryFromPrimitiveError(value)
    }
}

impl From<std::string::FromUtf8Error> for MCUtilsError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        MCUtilsError::UTF8Error(value)
    }
}

impl From<simd_cesu8::DecodingError> for MCUtilsError {
    fn from(value: simd_cesu8::DecodingError) -> Self {
        MCUtilsError::JavaStringDecodingError(value)
    }
}

impl std::fmt::Display for MCUtilsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MCUtilsError::DEFAULT => f.write_str("error"),
            MCUtilsError::IO(error) => f.write_fmt(format_args!("IO Error: {error:?}")),
            MCUtilsError::InvalidOffset(x, z) => {
                f.write_fmt(format_args!("Invalid Offset: x: {x}, z: {z}"))
            }
            MCUtilsError::UnknownCompression(val) => {
                f.write_fmt(format_args!("Unknown Compresssion sceme: {val}"))
            }
            MCUtilsError::TryFromPrimitiveError(try_from_primitive_error) => f.write_fmt(
                format_args!("Failed Primitive Conversion: {try_from_primitive_error:?}"),
            ),
            MCUtilsError::UTF8Error(utf_8_error) => f.write_fmt(format_args!(
                "Error creating string from bytes: {utf_8_error:?}"
            )),
            MCUtilsError::JavaStringDecodingError(cesu8_decoding_error) => f.write_fmt(
                format_args!("Error parsing a java string: {cesu8_decoding_error:?}"),
            ),
            MCUtilsError::ListError(len) => {
                f.write_fmt(format_args!("Error parsing list of len {len}"))
            }
            MCUtilsError::InvalidFile() => f.write_fmt(format_args!("invalid file maybeh")),
        }
    }
}

impl std::error::Error for MCUtilsError {}
