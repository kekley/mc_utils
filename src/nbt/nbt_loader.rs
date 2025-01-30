use std::{
    fs::{self, File},
    io::{BufReader, Read},
    sync::Arc,
};

use bytes::Bytes;
use lasso::ThreadedRodeo;

use crate::SpiderEyeError;

use super::nbt_compound::NBTCompound;

#[derive(Debug, Clone)]
pub struct NBTLoader {
    pub rodeo: Arc<ThreadedRodeo>,
}

impl NBTLoader {
    pub fn new() -> Self {
        Self {
            rodeo: Arc::new(ThreadedRodeo::new()),
        }
    }

    pub fn nbt_from_bytes(&self, bytes: &mut Bytes) -> Result<NBTCompound, SpiderEyeError> {
        NBTCompound::from_bytes(bytes, self.rodeo.clone())
    }
}
