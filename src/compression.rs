use std::io::{self, Write};

use byteorder::{BigEndian, ReadBytesExt};
use bytes::{buf, Buf, Bytes};
use num_enum::TryFromPrimitive;

use crate::spider_eye_error::SpiderEyeError;

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

pub fn decompress_bytes(
    data: &mut dyn Buf,
    compression_data: CompressionData,
) -> Result<Bytes, SpiderEyeError> {
    let mut take = data.take(compression_data.compressed_len as usize);

    match compression_data.scheme {
        CompressionScheme::Gzip => {
            let mut compressed_data = vec![0u8; compression_data.compressed_len as usize];
            let mut writer = flate2::write::GzDecoder::new(vec![]);
            Buf::copy_to_slice(&mut take, &mut compressed_data);
            writer.write_all(&compressed_data[..])?;
            Ok(writer.finish()?.into())
        }
        CompressionScheme::Zlib => {
            let mut compressed_data = vec![0u8; compression_data.compressed_len as usize];
            let mut writer = flate2::write::ZlibDecoder::new(vec![]);
            Buf::copy_to_slice(&mut take, &mut compressed_data);
            writer.write_all(&compressed_data[..])?;
            Ok(writer.finish()?.into())
        }
        CompressionScheme::Uncompressed => {
            let mut writer = vec![0u8; compression_data.compressed_len as usize];
            Buf::copy_to_slice(&mut take, &mut writer);
            Ok(writer.into())
        }
    }
}
