use std::{fs, sync::Arc};

use lasso::ThreadedRodeo;
use serde_json::Value;

use super::{multipart::MultiPart, variant::Variants};

#[derive(Debug, Clone)]
pub enum BlockStates {
    MultiPart(MultiPart),
    Variants(Variants),
}

impl BlockStates {
    pub fn new(path: &str, rodeo: &ThreadedRodeo) -> BlockStates {
        dbg!(path);
        let file = fs::read_to_string(path).expect("could not read file");
        let value: Value = serde_json::from_str(&file).expect("invalid json");
        if let Some(value) = value.get("variants") {
            BlockStates::Variants(Variants::from_json_value(value, &rodeo))
        } else if let Some(value) = value.get("multipart") {
            BlockStates::MultiPart(MultiPart::new(value, rodeo))
        } else {
            panic!()
        }
    }
}
