use std::{
    collections::HashMap,
    fs,
    io::{Cursor, Read, Seek},
    path::Path,
};

use crate::Region;

pub struct World {
    pub regions: HashMap<(i32, i32), Region>,
}

impl World {
    pub fn new(folder_path: &str) -> Self {
        let mut temp = Self {
            regions: HashMap::new(),
        };
        let read_dir = fs::read_dir(folder_path).expect("could not find folder");
        for dir in read_dir {
            let entry = dir.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension().unwrap() == "mca" {
                let file = fs::read(path).unwrap();
                let cursor = Cursor::new(file);
                let region: Region =
                    Region::from_stream(cursor).expect("error reading region file");
                temp.regions.insert((region.x, region.z), region);
            }
        }

        temp
    }

    pub fn get_region(&self, x: i32, z: i32) -> Option<&Region> {
        self.regions.get(&(x, z))
    }

    pub fn get_region_containing_chunk(&self, x: i32, z: i32) -> Option<&Region> {
        self.get_region(x >> 5, z >> 5)
    }

    pub fn get_region_containing_block(&self, x: i64, z: i64) -> Option<&Region> {
        self.get_region_containing_chunk((x >> 4) as i32, (z >> 4) as i32)
    }
}
