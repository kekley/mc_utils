use std::{
    fs,
    io::{Cursor, Read},
};

use compression::CompressionData;
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
    let mut player_file = fs::File::open("./132f1ee7-c4a2-48bc-808d-136271e4093f.dat")?;
    let mut bytes = Vec::with_capacity(player_file.metadata().unwrap().len() as usize);
    player_file.read_to_end(&mut bytes).unwrap();
    println!("{:?}", bytes);
    let mut cursor = Cursor::new(&mut bytes[..]);

    let player = NBTCompound::from_compressed_stream(
        &mut cursor,
        CompressionData {
            scheme: compression::CompressionScheme::Gzip,
            compressed_len: player_file.metadata().unwrap().len() as u32,
        },
    )?;
    println!("{}", player.children.len());
    let a = player.as_indented_string(0);
    println!("{}", a);
    Ok(())
}
