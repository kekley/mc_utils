use std::sync::Arc;

use lasso::{Spur, ThreadedRodeo};

use super::block_models::BlockModel;

#[derive(Debug)]
pub struct ResourceLoader {
    rodeo: Arc<ThreadedRodeo<Spur>>,
}

impl ResourceLoader {
    pub fn load_model(&self, path: &str) -> BlockModel {
        BlockModel::load(path, self.rodeo.clone())
    }
    pub fn new() -> Self {
        Self {
            rodeo: Arc::new(ThreadedRodeo::new()),
        }
    }
}
