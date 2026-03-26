#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{error::Error, path::PathBuf, str::FromStr};

use mc_utils::resource_loader::LoadedResources;

pub fn main() -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from_str("./test_assets/assets/").unwrap();

    let _a = LoadedResources::load_resource_folder(&path)?;

    Ok(())
}
