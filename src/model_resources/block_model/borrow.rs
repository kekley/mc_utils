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
