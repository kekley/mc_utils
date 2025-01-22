use std::{fs, rc::Rc};

use anyhow::{anyhow, Context, Error, Ok};
use glam::{IVec3, Vec3};
use rayon::vec;
use serde_json::{value, Value};

pub enum BlockRotation {
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}
impl From<&Value> for BlockRotation {
    fn from(value: &Value) -> Self {
        todo!()
    }
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

#[derive(Debug, Clone, Copy)]
enum ElementAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy)]
enum FaceName {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

pub struct Uv {
    x1: i8,
    y1: i8,
    x2: i8,
    y2: i8,
}

impl TryFrom<&Value> for Uv {
    type Error = anyhow::Error;
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        Ok(Uv::from(parse_i8vec4(value.get("uv").context("no uv")?)?))
    }
}
impl From<[i8; 4]> for Uv {
    fn from(value: [i8; 4]) -> Self {
        Uv {
            x1: value[0],
            y1: value[1],
            x2: value[2],
            y2: value[3],
        }
    }
}
enum TextureVariable {
    Variable(String),
    ResourcePath(String),
}
pub struct Face {
    name: FaceName,
    uv: Option<Uv>,
    texture: TextureVariable,
    cullface: FaceName,
    texture_rotation: BlockRotation,
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

impl TryFrom<&str> for DisplayPosition {
    fn try_from(value: &str) -> Result<DisplayPosition, anyhow::Error> {
        match value {
            "gui" => Ok(DisplayPosition::Gui),
            "ground" => Ok(DisplayPosition::Ground),
            "fixed" => Ok(DisplayPosition::Fixed),
            "thirdperson_righthand" => Ok(DisplayPosition::ThirdPersonRightHand),
            "thirdperson_lefthand" => Ok(DisplayPosition::ThirdPersonLeftHand),
            "firstperson_righthand" => Ok(DisplayPosition::FirstPersonRightHand),
            "firstperson_lefthand" => Ok(DisplayPosition::FirstPersonLeftHand),
            "head" => Ok(DisplayPosition::Head),
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

impl TryFrom<(&str, &Value)> for BlockDisplay {
    type Error = anyhow::Error;

    fn try_from(value: (&str, &Value)) -> Result<Self, Self::Error> {
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
            .filter_map(|(name, value)| BlockDisplay::try_from((name.as_str(), value)).ok())
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

fn parse_i8vec3(value: &Value) -> Result<[i8; 3], anyhow::Error> {
    let result: [i8; 3] = value
        .as_array()
        .context("was not array")?
        .iter()
        .map(|value| value.as_i64().expect("not an integer") as i8)
        .collect::<Vec<i8>>()
        .try_into()
        .ok()
        .context("not len 3")?;
    Ok(result)
}
fn parse_i8vec4(value: &Value) -> Result<[i8; 4], anyhow::Error> {
    let result: [i8; 4] = value
        .as_array()
        .context("was not array")?
        .iter()
        .map(|value| value.as_i64().expect("not an integer") as i8)
        .collect::<Vec<i8>>()
        .try_into()
        .ok()
        .context("not len 4")?;
    Ok(result)
}

impl From<&str> for FaceName {
    fn from(value: &str) -> Self {
        match value {
            "down" => FaceName::Down,
            "up" => FaceName::Up,
            "north" => FaceName::North,
            "south" => FaceName::South,
            "west" => FaceName::West,
            "east" => FaceName::East,
            _ => panic!("invalid value for facename"),
        }
    }
}

impl TryFrom<(&str, &Value)> for Face {
    type Error = anyhow::Error;
    fn try_from(value: (&str, &Value)) -> Result<Face, Error> {
        let name = FaceName::try_from(value.0)?;
        let texture = TextureVariable::try_from(value.1)?;
        let uv = Uv::try_from(value.1);
        let cullface = if let Some(face) = value.1.get("cullface") {
            FaceName::try_from(face.as_str().expect("invalid cullface"))?
        } else {
            name
        };
        if let Some(rotation) = value.1.get("rotation"){
            BlockRotation::
        }else{
            BlockRotation::Zero
        }

        Ok(Face {
            name,
            uv,
            texture,
            cullface,
            texture_rotation: todo!(),
            tint_index,
        })
    }
}

impl TryFrom<&Value> for TextureVariable {
    type Error = anyhow::Error;
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let value = value.get("texture").context("no texture value in face")?;
        let string_val = value.as_str().expect("texture value was not string");
        let first_char = string_val
            .chars()
            .nth(0)
            .expect("texture string had length of 0");
        if string_val.len() == 1 {
            panic!("invalid texture variable")
        }
        if first_char == '#' {
            return Ok(TextureVariable::Variable(string_val.to_string()));
        } else {
            return Ok(TextureVariable::ResourcePath(string_val.to_string()));
        }
    }
}
