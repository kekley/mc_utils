use std::hash::Hash;

use tracing::info;

use super::{
    region::{LazyRegion, LoadedRegion},
    world_error::WorldError,
};

#[derive(Debug)]
pub struct World {
    region_folder: String,
}

impl World {
    pub(crate) fn new(folder_path: &str) -> Result<World, WorldError> {
        info!("Opening world folder");
        let temp = Self {
            region_folder: folder_path.to_owned(),
        };
        dbg!(folder_path);

        Ok(temp)
    }

    pub fn get_region_lazy(&self, region_coords: RegionCoords) -> Option<LazyRegion> {
        info!("Creating lazy region loader for region {region_coords:?}");
        let x = region_coords.x;
        let z = region_coords.z;
        let mut path_str = String::new();
        path_str.push_str(&self.region_folder);
        path_str.push_str(&format!("/r.{x}.{z}.mca"));
        let file_path = path_str;
        LazyRegion::new(&file_path).ok()
    }

    pub fn load_region(&self, region_coords: RegionCoords) -> Option<LoadedRegion> {
        info!("Creating loaded region for: {region_coords:?}");
        self.get_region_lazy(region_coords).map(|f| (&f).into())
    }
}
