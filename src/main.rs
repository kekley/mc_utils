#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{error::Error, path::PathBuf, str::FromStr};

use spider_eye::resource_loader::LoadedResources;

pub fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let path = PathBuf::from_str("./test_assets/assets/").unwrap();

    let a = LoadedResources::load_resource_folder(&path)?;

    println!("textures:");
    for b in a.textures {
        println!("{loc:?}", loc = b.0);
    }

    println!("blockstates:");
    for b in a.variants {
        println!("{loc:?}", loc = b.0);
    }

    println!("models:");
    for b in a.models {
        println!("{loc:?}", loc = b.0);
    }

    Ok(())
}
