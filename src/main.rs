use spider_eye::{MCResourceLoader, SpiderEyeError};

fn main() -> Result<(), SpiderEyeError> {
    let loader = MCResourceLoader::new();
    let world = loader.open_world("./test_world");

    Ok(())
}
