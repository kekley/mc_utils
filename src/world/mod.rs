use std::path::Path;
use std::{path::PathBuf, str::FromStr};

use std::error::Error;

use hashbrown::HashMap;

use crate::coords::region::RegionCoords;
use crate::region::owned::Region;

pub struct World {
    folder: PathBuf,
    regions: HashMap<RegionCoords, PathBuf>,
}

impl World {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let path = PathBuf::from_str(path)?;

        let read_dir = std::fs::read_dir(&path)?;

        let regions = read_dir
            .filter_map(|entry| {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    let extension = path.extension()?;
                    if extension != "mca" {
                        return None;
                    }
                    let name = path.file_stem()?.to_str()?;
                    let mut split = name.split(".");
                    let x: i64 = split.next()?.parse().ok()?;
                    let z: i64 = split.next()?.parse().ok()?;
                    let coords = RegionCoords { x, z };

                    return Some((coords, path));
                }
                None
            })
            .collect::<HashMap<_, _>>();

        Ok(World {
            folder: path,
            regions,
        })
    }
    pub fn folder(&self) -> &Path {
        self.folder.as_path()
    }
    pub fn load_region(&self, coords: RegionCoords) -> Option<Region> {
        let path = self.regions.get(&coords)?;
        let region_bytes = std::fs::read(path).ok()?;
        let region = Region::from_bytes(coords, region_bytes);

        Some(region)
    }
}
