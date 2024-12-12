use std::{
    fs,
    io::{Cursor, Read},
    sync::{Arc, RwLock},
};

use indexmap::IndexMap;
use spider_eye::{CompressionData, CompressionScheme, NBTCompound, Region, SpiderEyeError};

extern crate spider_eye;

fn main() -> Result<(), SpiderEyeError> {
    let mut palette = IndexMap::new();
    let a = Arc::new(RwLock::new(palette));
    let mut region = Region::from_file("./r.0.0.mca".to_string(), a.clone())?;
    let chunk = region.get_chunk(0, 0, a.clone()).unwrap();
    println!("{:?}", chunk);
    let mut player_file = fs::File::open("./132f1ee7-c4a2-48bc-808d-136271e4093f.dat")?;
    let mut bytes = Vec::with_capacity(player_file.metadata().unwrap().len() as usize);
    player_file.read_to_end(&mut bytes).unwrap();
    println!("{:?}", bytes);
    let mut cursor = Cursor::new(&mut bytes[..]);
    Ok(())
}
