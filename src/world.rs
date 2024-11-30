use std::{fs, path::Path};

use crate::Region;

pub struct World<S> {
    pub regions: Vec<Region<S>>,
}

impl<S> World<S> {
    pub fn new(folder_path: &str) -> Self {
        let mut temp = Self { regions: vec![] };

        let read_dir = fs::read_dir(folder_path).expect("could not find folder");
        for dir in read_dir {
            let entry = dir.unwrap();
            let path = entry.path();
            if !path.is_dir() {
                
            }
        }
        todo!()
    }
}
