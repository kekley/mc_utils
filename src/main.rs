use bumpalo::Bump;
use spider_eye::{MCResourceLoader, SpiderEyeError};

extern crate spider_eye;

fn main() -> Result<(), SpiderEyeError> {
    let loader = MCResourceLoader::new();
    let world = loader.open_world("./test_world").unwrap();

    Ok(())
}
