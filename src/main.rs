use std::{
    fs,
    io::{Cursor, Read},
};

use spider_eye::{CompressionData, CompressionScheme, NBTCompound, Region, SpiderEyeError};

extern crate spider_eye;

fn main() -> Result<(), SpiderEyeError> {
    let mut file = fs::File::open("./r.0.0.mca")?;
    let mut bytes = Vec::<u8>::with_capacity(file.metadata().unwrap().len() as usize);
    file.read_to_end(&mut bytes)?;
    let region = Region::from_bytes(bytes)?;

    let chunk = region.get_chunk(0, 0).unwrap();
    println!("{:?}", chunk);
    let mut player_file = fs::File::open("./132f1ee7-c4a2-48bc-808d-136271e4093f.dat")?;
    let mut bytes = Vec::with_capacity(player_file.metadata().unwrap().len() as usize);
    player_file.read_to_end(&mut bytes).unwrap();
    println!("{:?}", bytes);
    let mut cursor = Cursor::new(&mut bytes[..]);

    let player = NBTCompound::from_compressed_stream(
        &mut cursor,
        CompressionData {
            scheme: CompressionScheme::Gzip,
            compressed_len: player_file.metadata().unwrap().len() as u32,
        },
    )?;
    println!("{}", player.children.len());
    let a = player.as_indented_string(0);
    println!("{}", a);
    Ok(())
}
