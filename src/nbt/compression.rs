use std::io::Write;

use byteorder::{BigEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;

use crate::error::spider_eye_error::MCUtilsError;

#[derive(Debug, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum CompressionScheme {
    Gzip = 1,
    Zlib = 2,
    Uncompressed = 3,
}
///Struct for storing data related to chunk decompression
pub struct CompressionData {
    scheme: CompressionScheme,
    compressed_len: usize,
}

impl CompressionData {
    pub fn new(mut data: &[u8]) -> Result<Self, MCUtilsError> {
        let len = data.read_u32::<BigEndian>()?;
        let scheme = data.read_u8()?;
        let compression_data = Self {
            scheme: CompressionScheme::try_from(scheme)
                .map_err(|_err| MCUtilsError::UnknownCompression(scheme))?,
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

pub fn decompress_chunk(data: &[u8]) -> Result<Vec<u8>, MCUtilsError> {
    let compression_data = CompressionData::new(data)?;
    match compression_data.scheme {
        CompressionScheme::Gzip => {
            let data_slice = data
                .get(5..compression_data.compressed_len)
                .expect("Chunk data should follow");
            let mut writer =
                flate2::write::GzDecoder::new(Vec::with_capacity(compression_data.compressed_len));
            writer.write_all(data_slice)?;
            Ok(writer.finish()?)
        }
        CompressionScheme::Zlib => {
            let data_slice = data
                .get(5..compression_data.compressed_len)
                .expect("Chunk data should follow");
            let mut writer = flate2::write::ZlibDecoder::new(Vec::with_capacity(
                compression_data.compressed_len,
            ));

            let _ = writer
                .write_all(data_slice)
                .map_err(|err| eprintln!("{err}"));
            Ok(writer.finish()?)
        }
        CompressionScheme::Uncompressed => {
            let compression_header_removed =
                data.get(5..).expect("Compression data should be present");

            Ok(compression_header_removed.to_vec())
        }
    }
}
