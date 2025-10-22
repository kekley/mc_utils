use hashbrown::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) enum RawBlockVariants<'a> {
    #[serde(borrow)]
    #[serde(alias = "variants")]
    Variants(HashMap<&'a str, RawVariantType<'a>>),

    #[serde(alias = "multipart")]
    Multipart(Vec<RawCase<'a>>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum RawVariantType<'a> {
    #[serde(borrow)]
    SingleVariant(RawModelProperties<'a>),
    MultiVariant(Vec<RawModelProperties<'a>>),
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawModelProperties<'a> {
    #[serde(borrow)]
    pub(crate) model: &'a str,
    #[serde(default)]
    pub(crate) x: i32,
    #[serde(default)]
    pub(crate) y: i32,
    #[serde(default)]
    pub(crate) uvlock: bool,
    #[serde(default = "default_weight")]
    pub(crate) weight: i32,
}

fn default_weight() -> i32 {
    1
}

#[derive(Debug, Deserialize)]
pub struct RawCase<'a> {
    #[serde(borrow)]
    #[serde(default)]
    pub(crate) when: Option<WhenStruct<'a>>,
    pub(crate) apply: RawApply<'a>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum RawApply<'a> {
    #[serde(borrow)]
    Single(RawModelProperties<'a>),
    Many(Vec<RawModelProperties<'a>>),
}

#[derive(Debug, Deserialize)]
pub(crate) enum OrAnd<'a> {
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
    pub(crate) or_and: Option<OrAnd<'a>>,
    #[serde(flatten)]
    pub(crate) single_state: Option<HashMap<&'a str, &'a str>>,
}

impl<'a> WhenStruct<'a> {
    pub fn condition<'b>(&'b self) -> RawWhen<'a, 'b> {
        if let Some(or_and) = &self.or_and {
            match or_and {
                OrAnd::Or(hash_maps) => RawWhen::Or(hash_maps.as_slice()),
                OrAnd::And(hash_maps) => RawWhen::And(hash_maps.as_slice()),
            }
        } else if let Some(single_state) = &self.single_state {
            RawWhen::Single(single_state)
        } else {
            unreachable!()
        }
    }
}

pub enum RawWhen<'a, 'b> {
    Or(&'b [HashMap<&'a str, &'a str>]),
    And(&'b [HashMap<&'a str, &'a str>]),
    Single(&'b HashMap<&'a str, &'a str>),
}

#[cfg(test)]
mod tests {
    use crate::block_state::serde::RawBlockVariants;

    #[test]
    fn test_block_state() {
        let str =
            include_str!("../../../test_assets/assets/minecraft/blockstates/acacia_button.json");

        let a: Result<RawBlockVariants<'static>, serde_json::Error> = serde_json::de::from_str(str);

        let _b = a
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

                let a: Result<RawBlockVariants<'_>, serde_json::Error> =
                    serde_json::de::from_str(file.as_str());

                let _b = a.map_err(|err| eprintln!("{err:?}")).unwrap();
            }
        }
    }
}
