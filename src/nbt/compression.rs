use std::io::Write;

use crate::spider_eye_error::SpiderEyeError;
use byteorder::{BigEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;

#[derive(Debug, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum CompressionScheme {
    Gzip = 1,
    Zlib = 2,
    Uncompressed = 3,
}

pub struct CompressionData {
    scheme: CompressionScheme,
    compressed_len: usize,
}

impl CompressionData {
    pub fn new(mut data: &[u8]) -> Result<Self, SpiderEyeError> {
        let len = data.read_u32::<BigEndian>()?;
        let scheme = data.read_u8()?;
        let compression_data = Self {
            scheme: CompressionScheme::try_from(scheme)
                .map_err(|_| SpiderEyeError::UnknownCompression(scheme))?,
            compressed_len: (len - 1) as usize,
        };

        Ok(compression_data)
    }
    pub fn scheme(&self) -> CompressionScheme {
        self.scheme
    }
    pub fn compressed_len(&self) -> usize {
        self.compressed_len
    }
}

pub fn decompress_bytes(mut data: Vec<u8>) -> Result<Vec<u8>, SpiderEyeError> {
    let compression_data = CompressionData::new(&data)?;
    match compression_data.scheme {
        CompressionScheme::Gzip => {
            let data_slice = &data.as_slice()[5..];
            let mut writer =
                flate2::write::GzDecoder::new(Vec::with_capacity(compression_data.compressed_len));
            writer.write_all(&data_slice)?;
            Ok(writer.finish()?)
        }
        CompressionScheme::Zlib => {
            let data_slice = &data.as_slice()[5..];
            let mut writer = flate2::write::ZlibDecoder::new(Vec::with_capacity(
                compression_data.compressed_len,
            ));
            writer.write_all(&data_slice)?;
            Ok(writer.finish()?)
        }
        CompressionScheme::Uncompressed => {
            data.rotate_left(5);
            data.pop();
            data.pop();
            data.pop();
            data.pop();
            data.pop();
            Ok(data)
        }
    }
}
