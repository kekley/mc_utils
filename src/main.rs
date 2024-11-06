use std::{
    fs,
    io::{Cursor, Read},
};

use nbt_compound::NBTCompound;
use region::Region;
use spider_eye_error::SpiderEyeError;

mod compression;
mod nbt_compound;
mod nbt_ids;
mod nbt_tag;
mod region;
mod spider_eye_error;

fn main() -> Result<(), SpiderEyeError> {
    let mut file = fs::File::open("./r.0.0.mca")?;
    let mut byte_stream = Vec::<u8>::with_capacity(file.metadata().unwrap().len() as usize);
    file.read_to_end(&mut byte_stream)?;
    let mut cursor = Cursor::new(&mut byte_stream[..]);
    let mut region = Region::from_stream(&mut cursor)?;

    let first_segment = region.chunk_segments[0];

    let mut chunk_data = region.read_chunk_from_segment(first_segment)?;
    fs::write("./chunk_data.dat", &chunk_data)?;
    let mut cursor = Cursor::new(&mut chunk_data);
    let compound = NBTCompound::from_borrowed_stream(&mut cursor)?;
    let str = compound.as_indented_string(0);
    fs::write("./parsed_nbt.txt", str)?;
    Ok(())
}
