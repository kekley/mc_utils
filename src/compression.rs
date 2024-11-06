use byteorder::{BigEndian, ReadBytesExt};
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};

use crate::{nbt_ids::NBTId, spider_eye_error::SpiderEyeError};

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum CompressionScheme {
    Gzip = 1,
    Zlib = 2,
    Uncompressed = 3,
}

pub struct CompressionData {
    pub scheme: CompressionScheme,
    pub compressed_len: u32,
}

impl CompressionData {
    pub fn new(mut data: &[u8]) -> Result<Self, SpiderEyeError> {
        println!("{:?}", &data[..]);

        let len = data.read_u32::<BigEndian>()?;
        let scheme = data.read_u8()?;
        let compression_data = Self {
            scheme: CompressionScheme::try_from(scheme)
                .map_err(|_| SpiderEyeError::UnknownCompression(scheme))?,
            compressed_len: len - 1,
        };

        Ok(compression_data)
    }
}
