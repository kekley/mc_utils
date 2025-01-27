use std::{
    fs,
    io::{Cursor, Read},
    sync::{Arc, RwLock},
};

use spider_eye::{
    block_models::BlockModel, CompressionData, CompressionScheme, NBTCompound, Region,
    SpiderEyeError,
};

extern crate spider_eye;

fn main() -> Result<(), SpiderEyeError> {
    let button = BlockModel::load("./test_assets/assets/minecraft/models/block/acacia_button.json");
    dbg!(&button);
    Ok(())
}
