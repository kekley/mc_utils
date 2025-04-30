use spider_eye::{
    loaded_world::WorldCoords, resource::BlockStates, MCResourceLoader, SpiderEyeError,
};

extern crate spider_eye;
macro_rules! dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `println!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `println!`
    // will be malformed.
    () => {
        ::std::println!("[{}:{}]", ::std::file!(), ::std::line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                ::std::println!("[{}:{}] {} = {:#?}",
                    ::std::file!(), ::std::line!(), ::std::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($(::std::dbg!($val)),+,)
    };
}

fn main() -> Result<(), SpiderEyeError> {
    let loader = MCResourceLoader::new();
    let world = loader.open_world("./test_world");
    let block = world
        .get_block(&WorldCoords { x: 0, y: -64, z: 0 })
        .unwrap();
    let states = loader.load_block_states("minecraft:redstone_wire");
    dbg!(states);
    let a = loader.load_models(&block);

    Ok(())
}
