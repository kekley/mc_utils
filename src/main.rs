use std::{
    fs,
    io::{Cursor, Read},
    os::windows::fs::MetadataExt,
};

use region::{Region, RegionError};

mod nbt_compound;
mod nbt_ids;
mod nbt_tag;
mod region;

fn main() -> Result<(), RegionError> {
    fastnbt::from_bytes(input)
    let mut file = fs::File::open("./r.0.0.mca")?;
    let mut byte_stream = Vec::<u8>::with_capacity(file.metadata().unwrap().len() as usize);
    file.read_to_end(&mut byte_stream)?;
    let mut cursor = Cursor::new(&mut byte_stream[..]);
    let mut region = Region::from_stream(&mut cursor)?;

    let first_segment = region.chunk_segments[0];

    let chunk_data = region.read_chunk_from_segment(first_segment)?;
    fs::write("./chunk_data.dat", chunk_data)?;
    Ok(())
}
