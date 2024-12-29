use std::fs;

use serde_json::Value;

enum Faces {
    Up,
    Down,
    North,
    South,
    East,
    West,
}

pub struct BlockModel {}

impl BlockModel {
    pub fn from_json(path: &str) -> Self {
        let json = fs::read_to_string(path).expect("failed to read json file");
        let val: Value = serde_json::from_str(&json).expect("failed to parse json");
        let elements = val
            .get("elements")
            .expect("No elements")
            .as_array()
            .expect("Not a valid block model");

        for element in elements {
            println!("{}", element);
        }

        todo!()
    }
}

fn parse_element(val: Value) {}
