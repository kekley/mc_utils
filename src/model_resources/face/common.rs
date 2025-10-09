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
