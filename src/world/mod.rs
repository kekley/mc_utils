use std::{path::PathBuf, str::FromStr};

use std::error::Error;

pub struct World {
    folder: PathBuf,
    regions: Vec<PathBuf>,
}

impl World {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let path = PathBuf::from_str(path)?;

        let read_dir = std::fs::read_dir(path)?;

        todo!()
    }
}
