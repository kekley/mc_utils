use std::{fs, rc::Rc};

use anyhow::{anyhow, Context, Ok};
use glam::{IVec3, Vec3};
use rayon::vec;
use serde_json::{value, Value};

pub enum BlockRotation {
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}

pub struct Variant {}

pub struct MultiPart {}

pub struct BlockElement {
    from: [i8; 3],
    to: [i8; 3],
    rotation: ElementRotation,
    shade: bool,
    faces: [Option<FaceName>; 6],
}

enum ElementAxis {
    X,
    Y,
    Z,
}

enum FaceName {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

pub struct Uv {
    x1: u8,
    y1: u8,
    x2: u8,
    y2: u8,
}
enum TextureVariable {
    Variable,
    ResourcePath,
}
pub struct Face {
    name: FaceName,
    uv: Uv,
    texture: TextureVariable,
    cullface: FaceName,
    rotation: BlockRotation,
    tint_index: i32,
}
pub struct ElementRotation {
    origin: [i8; 3],
    axis: ElementAxis,
    angle: f32,
    rescale: bool,
}

enum DisplayPosition {
    ThirdPersonRightHand,
    ThirdPersonLeftHand,
    FirstPersonRightHand,
    FirstPersonLeftHand,
    Gui,
    Head,
    Ground,
    Fixed,
}

impl TryFrom<&String> for DisplayPosition {
    fn try_from(value: &String) -> Result<DisplayPosition, anyhow::Error> {
        match value.as_str() {
            "gui" => Ok(DisplayPosition::Gui),
            "ground" => Ok(DisplayPosition::Ground),
            "fixed" => Ok(DisplayPosition::Fixed),
            "thirdperson_righthand" => Ok(DisplayPosition::ThirdPersonRightHand),
            "thirdperson_lefthand" => Ok(DisplayPosition::ThirdPersonLeftHand),
            "firstperson_righthand" => Ok(DisplayPosition::FirstPersonRightHand),
            "firstperson_lefthand" => Ok(DisplayPosition::FirstPersonLeftHand),
            _ => Err(anyhow!("invalid display enum")),
        }
    }

    type Error = anyhow::Error;
}
pub struct BlockDisplay {
    position: DisplayPosition,
    rotation: Vec3,
    translation: Vec3,
    scale: Vec3,
}

fn parse_vec3(data: &Value) -> Result<Vec3, anyhow::Error> {
    let result: [f32; 3] = data
        .as_array()
        .context("rotation was not array")?
        .iter()
        .map(|value| value.as_f64().expect("not float") as f32)
        .collect::<Vec<f32>>()
        .try_into()
        .ok()
        .context("not len 3")?;
    Ok(Vec3::from_slice(&result))
}

impl TryFrom<(&String, &Value)> for BlockDisplay {
    type Error = anyhow::Error;

    fn try_from(value: (&String, &Value)) -> Result<Self, Self::Error> {
        let position = value.0;
        let data = value.1;
        let rotation = parse_vec3(data.get("rotation").context("no rotation")?)?;
        let translation = parse_vec3(data.get("translation").context("no translation")?)?;
        let scale = parse_vec3(data.get("scale").context("no scale")?)?;
        let res = BlockDisplay {
            position: position.try_into()?,
            rotation,
            translation,
            scale,
        };

        Ok(res)
    }
}

pub struct BlockModel {
    parent: Option<String>,
    ambient_occlusion: bool,
    displays: Vec<BlockDisplay>,
    particle: TextureVariable,
    textures: Vec<TextureVariable>,
    elements: Vec<BlockElement>,
}

pub enum BlockStates {
    Variant(Vec<Variant>),
    MultiPart(),
}

pub struct BlockVariant<'a> {
    name: &'a str,
    model: BlockModel,
}

const MODEL_PATH: &str = "./assets/default_resource_pack/assets/minecraft/models/";
impl BlockModel {
    pub fn from_json(path: &str) -> BlockModel {
        let json = fs::read_to_string(path).expect("failed to read json file");
        let model: Value = serde_json::from_str(&json).expect("failed to parse json");
        let result = BlockModel {
            parent: todo!(),
            ambient_occlusion: todo!(),
            displays: todo!(),
            particle: todo!(),
            textures: todo!(),
            elements: todo!(),
        };
    }
}

fn parse_parent(value: &Value) -> Option<String> {
    let parent = value.get("parent")?;

    Some(parent.as_str().expect("parent was not string").to_owned())
}

fn parse_ambient_occlusion(value: &Value) -> bool {
    let ambient_occlusion_value = value.get("ambientocclusion");

    match ambient_occlusion_value {
        Some(value) => value.as_bool().expect("ambient occlusion was not bool"),
        None {} => true,
    }
}

fn parse_display(value: &Value) -> Vec<BlockDisplay> {
    let display_value = value.get("display");

    if let Some(display) = display_value {
        display
            .as_object()
            .expect("display was not valid")
            .iter()
            .filter_map(|(name, value)| BlockDisplay::try_from((name, value)).ok())
            .collect::<Vec<BlockDisplay>>()
    } else {
        return vec![];
    }
}

fn parse_elements(value: &Value) -> Vec<BlockElement> {
    let elements = value.get("elements");
    if let Some(elements) = elements {
        todo!()
    } else {
        return vec![];
    }
}

fn parse_faces(value: &Value) -> [Option<Face>; 6] {
    const NONE_VALUE: Option<Face> = None;
    let faces = value.get("faces");
    if let Some(faces) = faces {
        let obj = faces.as_object().expect("faces obj not valid");
        
        todo!()
    } else {
        return [NONE_VALUE; 6];
    }
}

fn parse_i8vec3(value: &Value) -> [i8; 3] {
    todo!()
}
