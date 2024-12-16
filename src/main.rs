use std::{
    fs,
    io::{Cursor, Read},
    sync::{Arc, RwLock},
};

use indexmap::IndexMap;
use spider_eye::{
    BlockModel, CompressionData, CompressionScheme, NBTCompound, Region, SpiderEyeError,
};

extern crate spider_eye;

fn main() -> Result<(), SpiderEyeError> {
    BlockModel::from_json("./cube.json");
    Ok(())
}
