use std::{error::Error, path::PathBuf, str::FromStr, time::Instant};

use spider_eye::{
    borrow::nbt_compound::RootNBTCompound, chunk::borrow::Chunk, region::borrow::Region,
    resource_loader::load_resource_folder, section::borrow::Section,
};

pub fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let path = PathBuf::from_str("./test_assets/assets/").unwrap();

    let _ = load_resource_folder(&path);
    Ok(())
}
