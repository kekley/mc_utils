use std::fmt::Display;

///A Rotation at the block level that only works in 90 degree increments
#[derive(Debug, Clone, Copy)]
pub enum BlockRotation {
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}

#[derive(Debug, Clone, Copy)]
pub enum FromI32Error {
    InvalidValue,
}

impl Display for FromI32Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl std::error::Error for FromI32Error {}

impl TryFrom<i32> for BlockRotation {
    type Error = FromI32Error;

    #[inline]
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value % 360 {
            0 => Ok(BlockRotation::Zero),
            90 => Ok(BlockRotation::Ninety),
            180 => Ok(BlockRotation::OneEighty),
            270 => Ok(BlockRotation::TwoSeventy),
            _ => Err(FromI32Error::InvalidValue),
        }
    }
}
impl TryFrom<&i32> for BlockRotation {
    type Error = FromI32Error;

    #[inline]
    fn try_from(value: &i32) -> Result<Self, Self::Error> {
        match *value % 360 {
            0 => Ok(BlockRotation::Zero),
            90 => Ok(BlockRotation::Ninety),
            180 => Ok(BlockRotation::OneEighty),
            270 => Ok(BlockRotation::TwoSeventy),
            _ => Err(FromI32Error::InvalidValue),
        }
    }
}
