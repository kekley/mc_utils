use hashbrown::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum BlockStateType<'a> {
    #[serde(borrow)]
    #[serde(alias = "variants")]
    Variants(HashMap<&'a str, RawVariantType<'a>>),

    #[serde(alias = "multipart")]
    Multipart(Vec<Case<'a>>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RawVariantType<'a> {
    #[serde(borrow)]
    SingleVariant(ModelProperties<'a>),
    MultiVariant(Vec<ModelProperties<'a>>),
}

#[derive(Debug, Deserialize)]
pub(crate) struct ModelProperties<'a> {
    #[serde(borrow)]
    model: &'a str,
    #[serde(default)]
    x: i32,
    #[serde(default)]
    y: i32,
    #[serde(default)]
    uvlock: bool,
    #[serde(default = "default_weight")]
    weight: i32,
}

impl<'a> ModelProperties<'a> {
    pub fn model(&self) -> &'a str {
        self.model
    }
    pub fn x_rotation(&self) -> i32 {
        self.x
    }
    pub fn y_rotation(&self) -> i32 {
        self.y
    }

    pub fn uvlock(&self) -> bool {
        self.uvlock
    }
    pub fn weight(&self) -> i32 {
        self.weight
    }
}

fn default_weight() -> i32 {
    1
}

#[derive(Debug, Deserialize)]
pub struct Case<'a> {
    #[serde(borrow)]
    #[serde(default)]
    when: Option<WhenStruct<'a>>,
    apply: Apply<'a>,
}

impl<'a> Case<'a> {
    pub fn when(&self) -> Option<&WhenStruct<'a>> {
        self.when.as_ref()
    }
    pub fn apply(&self) -> &Apply<'a> {
        &self.apply
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Apply<'a> {
    #[serde(borrow)]
    Single(ModelProperties<'a>),
    Many(Vec<ModelProperties<'a>>),
}

#[derive(Debug, Deserialize)]
enum OrAnd<'a> {
    #[serde(borrow)]
    #[serde(alias = "OR")]
    Or(Vec<HashMap<&'a str, &'a str>>),

    #[serde(alias = "AND")]
    And(Vec<HashMap<&'a str, &'a str>>),
}

#[derive(Deserialize, Debug)]
pub struct WhenStruct<'a> {
    #[serde(flatten)]
    #[serde(borrow)]
    or_and: Option<OrAnd<'a>>,
    #[serde(flatten)]
    single_state: Option<HashMap<&'a str, &'a str>>,
}

impl<'a> WhenStruct<'a> {
    pub fn condition<'b>(&'b self) -> When<'a, 'b> {
        if let Some(or_and) = &self.or_and {
            match or_and {
                OrAnd::Or(hash_maps) => When::Or(hash_maps.as_slice()),
                OrAnd::And(hash_maps) => When::And(hash_maps.as_slice()),
            }
        } else if let Some(single_state) = &self.single_state {
            When::Single(single_state)
        } else {
            unreachable!()
        }
    }
}

pub enum When<'a, 'b> {
    Or(&'b [HashMap<&'a str, &'a str>]),
    And(&'b [HashMap<&'a str, &'a str>]),
    Single(&'b HashMap<&'a str, &'a str>),
}

#[cfg(test)]
mod tests {
    use crate::block_state::serde::BlockStateType;

    #[test]
    fn test_block_state() {
        let str =
            include_str!("../../../test_assets/assets/minecraft/blockstates/acacia_button.json");

        let a: Result<BlockStateType<'static>, serde_json::Error> = serde_json::de::from_str(str);

        let b = a
            .map_err(|err| {
                eprintln!("{err}:?");
            })
            .unwrap();
    }
    #[test]
    fn test_mc_blockstates() {
        let read_dir = std::fs::read_dir("./test_assets/assets/minecraft/blockstates/").unwrap();

        for entry in read_dir.flatten() {
            let path = entry.path();

            if path.is_file() && path.extension().unwrap() == "json" {
                let file = std::fs::read_to_string(path).unwrap();

                let a: Result<BlockStateType<'_>, serde_json::Error> =
                    serde_json::de::from_str(file.as_str());

                let b = a.map_err(|err| eprintln!("{err:?}")).unwrap();
            }
        }
    }
}
