use std::{
    collections::HashMap,
    fs,
    io::{Cursor, Read, Seek},
    path::Path,
};

use smol_str::SmolStr;

use crate::{ChunkData, Region};

pub struct World<'a> {
    pub regions: HashMap<(i32, i32), Region<'a>>,
    pub loaded_chunks: HashMap<(i32, i32), ChunkData>,
    pub global_palette: Vec<SmolStr>,
}

impl<'a> World<'a> {
    pub fn new(folder_path: &str) -> Self {
        let mut temp = Self {
            regions: HashMap::new(),
            loaded_chunks: HashMap::new(),
            global_palette: vec![],
        };
        let read_dir = fs::read_dir(folder_path).expect("could not find folder");
        for dir in read_dir {
            let entry = dir.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension().unwrap() == "mca" {
                let mut file = fs::File::open(path).unwrap();
                let region: Region =
                    Region::from_stream(&mut file).expect("error reading region file");
                temp.regions.insert((region.x, region.z), region);
            }
        }

        temp
    }

    fn get_region(&'a self, x: i32, z: i32) -> Option<&Region> {
        self.regions.get(&(x, z))
    }

    fn get_region_containing_chunk(&'a self, x: i32, z: i32) -> Option<&Region> {
        self.get_region(x >> 5, z >> 5)
    }

    fn get_region_containing_block(&'a self, x: i64, z: i64) -> Option<&Region> {
        self.get_region_containing_chunk((x >> 4) as i32, (z >> 4) as i32)
    }
}
