use std::{
    fs,
    io::{Cursor, Read},
    sync::{Arc, RwLock},
};

use indexmap::IndexMap;
use spider_eye::{
    block_models::BlockModel, CompressionData, CompressionScheme, NBTCompound, Region,
    SpiderEyeError,
};

extern crate spider_eye;

fn main() -> Result<(), SpiderEyeError> {
    let cube = BlockModel::from_json("./cube.json");
    dbg!(cube);
    Ok(())
}
