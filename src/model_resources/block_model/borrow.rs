use hashbrown::HashMap;

use crate::{
    block_model::common::{DisplayPosition, PositionData},
    element::borrow::Element,
};

#[derive(Debug, Clone)]
pub struct BlockModel<'a> {
    pub parent: Option<&'a str>,
    pub ambient_occlusion: bool,
    pub display: HashMap<DisplayPosition, PositionData>,
    pub textures: HashMap<&'a str, &'a str>,
    pub elements: Vec<Element<'a>>,
}

impl BlockModel<'_> {
    pub fn get_parent(&self) -> Option<&str> {
        self.parent
    }

    pub fn get_ambient_occlusion(&self) -> bool {
        self.ambient_occlusion
    }

    pub fn get_display(&self) -> &HashMap<DisplayPosition, PositionData> {
        &self.display
    }

    pub fn get_textures(&self) -> &HashMap<&str, &str> {
        &self.textures
    }

    pub fn get_elements(&self) -> &[Element<'_>] {
        &self.elements
    }
}
