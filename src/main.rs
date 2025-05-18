use spider_eye::{
    loaded_world::WorldCoords, resource::BlockStates, MCResourceLoader, SpiderEyeError,
};

extern crate spider_eye;

fn main() -> Result<(), SpiderEyeError> {
    let loader = MCResourceLoader::new();
    let world = loader.open_world("./test_world").unwrap();

    Ok(())
}
