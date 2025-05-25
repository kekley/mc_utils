use std::io::Write;

use crate::spider_eye_error::SpiderEyeError;
use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;
use byteorder::{BigEndian, ReadBytesExt};
use bytes::{Buf, Bytes};
use num_enum::TryFromPrimitive;

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum CompressionScheme {
    Gzip = 1,
    Zlib = 2,
    Uncompressed = 3,
}

pub struct CompressionData {
    pub scheme: CompressionScheme,
    pub compressed_len: usize,
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
}

pub fn decompress_bytes<'a>(
    data: &mut dyn Buf,
    compression_data: CompressionData,
    bump: &'a mut Bump,
) -> Result<BumpVec<'a, u8>, SpiderEyeError> {
    let mut take = data.take(compression_data.compressed_len as usize);

    match compression_data.scheme {
        CompressionScheme::Gzip => {
            let mut compressed_data_buffer =
                bump.alloc_slice_fill_default(compression_data.compressed_len as usize);
            let mut writer = flate2::write::GzDecoder::new(BumpVec::new_in(bump));
            Buf::copy_to_slice(&mut take, &mut compressed_data_buffer);
            writer.write_all(&compressed_data_buffer[..])?;
            Ok(writer.finish()?)
        }
        CompressionScheme::Zlib => {
            let mut compressed_data_buffer =
                bump.alloc_slice_fill_default(compression_data.compressed_len as usize);
            let mut writer = flate2::write::ZlibDecoder::new(BumpVec::new_in(bump));
            Buf::copy_to_slice(&mut take, &mut compressed_data_buffer);
            writer.write_all(&compressed_data_buffer[..])?;
            Ok(writer.finish()?.into())
        }
        CompressionScheme::Uncompressed => {
            let mut writer =
                BumpVec::with_capacity_in(compression_data.compressed_len as usize, bump);
            Buf::copy_to_slice(&mut take, &mut writer);
            Ok(writer.into())
        }
    }
}
