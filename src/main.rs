#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{error::Error, path::PathBuf, str::FromStr};

use spider_eye::resource_loader::ResourceLoader;

pub fn main() -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from_str("./test_assets/assets/").unwrap();

    let _a = ResourceLoader::load_resource_folder(&path)?;

    Ok(())
}
