pub mod face_name {
    use serde::Deserialize;

    #[derive(Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy)]
    pub enum FaceName {
        #[serde(alias = "down")]
        Down = 2,

        #[serde(alias = "up")]
        Up = 3,

        #[serde(alias = "north")]
        North = 4,

        #[serde(alias = "south")]
        South = 5,

        #[serde(alias = "west")]
        West = 0,

        #[serde(alias = "east")]
        East = 1,
    }
    impl FaceName {
        pub fn from_usize(num: usize) -> Option<FaceName> {
            match num {
                0 => Some(FaceName::West),
                1 => Some(FaceName::East),
                2 => Some(FaceName::Up),
                3 => Some(FaceName::Down),
                4 => Some(FaceName::North),
                5 => Some(FaceName::South),
                _ => None,
            }
        }
    }
}

pub mod rotation {
    use serde::Deserialize;

    #[derive(Deserialize, Debug, Clone)]
    pub struct Rotation {
        origin: [f32; 3],
        axis: Axis,
        angle: f32,
        #[serde(default)]
        rescale: bool,
    }

    impl Rotation {
        pub fn origin(&self) -> &[f32; 3] {
            &self.origin
        }
        pub fn axis(&self) -> Axis {
            self.axis
        }
        pub fn angle(&self) -> f32 {
            self.angle
        }
        pub fn rescale(&self) -> bool {
            self.rescale
        }
    }

    #[derive(Deserialize, Debug, Clone, Copy)]
    pub enum Axis {
        #[serde(alias = "x")]
        X,

        #[serde(alias = "y")]
        Y,

        #[serde(alias = "z")]
        Z,
    }
}
