use num_enum::TryFromPrimitive;

pub const END_ID: u8 = 0;
pub const BYTE_ID: u8 = 1;
pub const SHORT_ID: u8 = 2;
pub const INT_ID: u8 = 3;
pub const LONG_ID: u8 = 4;
pub const FLOAT_ID: u8 = 5;
pub const DOUBLE_ID: u8 = 6;
pub const BYTE_ARRAY_ID: u8 = 7;
pub const STRING_ID: u8 = 8;
pub const LIST_ID: u8 = 9;
pub const COMPOUND_ID: u8 = 10;
pub const INT_ARRAY_ID: u8 = 11;
pub const LONG_ARRAY_ID: u8 = 12;

#[derive(Debug, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum NBTId {
    EndId = 0,
    ByteId = 1,
    ShortId = 2,
    IntId = 3,
    LongId = 4,
    FloatId = 5,
    DoubleId = 6,
    ByteArrayId = 7,
    StringId = 8,
    ListId = 9,
    CompoundId = 10,
    IntArrayId = 11,
    LongArrayId = 12,
}
