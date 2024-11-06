use std::{
    fs,
    io::{Cursor, Read},
};

use nbt_compound::NBTCompound;
use region::Region;
use spider_eye_error::SpiderEyeError;

mod chunk;
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

    let chunk = region.get_chunk(0, 0).unwrap();
    println!("{}", chunk.data.as_indented_string(0));
    Ok(())
}
