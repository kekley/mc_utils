pub mod face_name {

    use serde::Deserialize;

    use crate::block_state::common::BlockRotation;

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
        #[inline]
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
        pub fn iter_faces() -> impl Iterator<Item = FaceName> {
            FaceNameIter::default()
        }

        pub fn rotate_x(self, rotation: BlockRotation) -> Self {
            match rotation {
                BlockRotation::Zero => self,
                BlockRotation::Ninety => match self {
                    FaceName::Up => FaceName::North,
                    FaceName::North => FaceName::Down,
                    FaceName::Down => FaceName::South,
                    FaceName::South => FaceName::Up,
                    _ => self, // East and West are unaffected by X rotation
                },
                BlockRotation::OneEighty => match self {
                    FaceName::Up => FaceName::Down,
                    FaceName::North => FaceName::South,
                    FaceName::Down => FaceName::Up,
                    FaceName::South => FaceName::North,
                    _ => self,
                },
                BlockRotation::TwoSeventy => match self {
                    FaceName::Up => FaceName::South,
                    FaceName::North => FaceName::Up,
                    FaceName::Down => FaceName::North,
                    FaceName::South => FaceName::Down,
                    _ => self,
                },
            }
        }
    }

    #[derive(Default, Debug)]
    pub struct FaceNameIter {
        offset: usize,
    }
    impl Iterator for FaceNameIter {
        type Item = FaceName;
        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            let offset = self.offset;
            self.offset += 1;
            if offset > 5 {
                None
            } else {
                FaceName::from_usize(offset)
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
