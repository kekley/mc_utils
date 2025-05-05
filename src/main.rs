use spider_eye::{
    loaded_world::WorldCoords, resource::BlockStates, MCResourceLoader, SpiderEyeError,
};

extern crate spider_eye;

fn main() -> Result<(), SpiderEyeError> {
    let loader = MCResourceLoader::new();
    let world = loader.open_world("./test_world");
    let block = world
        .get_block(&WorldCoords { x: 0, y: -64, z: 0 })
        .unwrap();
    let states = loader
        .load_block_states_str("minecraft:redstone_wire")
        .unwrap();
    let a = loader.load_variants_for(&block, &states);

    Ok(())
}
