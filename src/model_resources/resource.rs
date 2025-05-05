use std::{fs, sync::Arc};

use lasso::ThreadedRodeo;
use serde_json::Value;

use crate::MCResourceLoader;

use super::{multipart::Multipart, variant::Variants};

#[derive(Debug, Clone)]
pub enum BlockStates {
    MultiPart(Multipart),
    Variants(Variants),
}

impl BlockStates {
    pub fn new(path: &str, loader: &MCResourceLoader) -> Option<BlockStates> {
        dbg!(path);
        let file = fs::read_to_string(path).ok();
        if file.is_none() {
            dbg!("blockstate file not found", path);
        }
        let value: Value = serde_json::from_str(&file?).expect("invalid json");
        if let Some(value) = value.get("variants") {
            Some(BlockStates::Variants(Variants::from_json_value(
                value, &loader,
            )))
        } else if let Some(value) = value.get("multipart") {
            Some(BlockStates::MultiPart(Multipart::new(value, loader)))
        } else {
            dbg!("Not a valid variant or multipart");
            return None;
        }
    }
}
