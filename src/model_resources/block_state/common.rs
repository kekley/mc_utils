use std::{fmt::Display, str, sync::RwLock};

use bumpalo::Bump;
use hashbrown::HashSet;

///A Rotation at the block level that only works in 90 degree increments
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BlockRotation {
    #[default]
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}
impl BlockRotation {
    pub fn to_degrees(self) -> f32 {
        match self {
            BlockRotation::Zero => 0.0,
            BlockRotation::Ninety => 90.0,
            BlockRotation::OneEighty => 180.0,
            BlockRotation::TwoSeventy => 270.0,
        }
    }
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

#[derive(Debug, Default)]
///A bump allocated string interner that gives out string slices with the lifetime of the interner
pub(crate) struct UniqueStrings {
    bump: Bump,
    strings: RwLock<HashSet<&'static str>>,
}

impl UniqueStrings {
    pub(crate) fn new() -> Self {
        Default::default()
    }
    ///Returns a reference to an arena allocated string slice
    pub(crate) fn get_or_intern<'a>(&'a self, new_str: &str) -> &'a str {
        (self.get_or_insert_inner(new_str)) as _
    }

    #[expect(unsafe_code)]
    fn get_or_insert_inner<'a>(&'a self, new_str: &str) -> &'a str {
        let interned_str = *self
            .strings
            .write()
            .unwrap()
            .get_or_insert_with(new_str, |str| {
                let new_alloc = self.bump.alloc_str(str);

                //SAFETY The memory of this slice will remain valid for the duration of this UniqueStrings and all references handed out have a lifetime tied to UniqueStrings
                let static_slice =
                    unsafe { std::slice::from_raw_parts(new_alloc.as_ptr(), new_alloc.len()) };

                //SAFETY these bytes came from a str, so they are always valid utf8
                unsafe { str::from_utf8_unchecked(static_slice) }
            });

        interned_str
    }
}
