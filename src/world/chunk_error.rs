#[derive(Debug)]

pub struct ChunkError {
    pub kind: ChunkErrorKind,
}
#[derive(Debug)]

pub enum ChunkErrorKind {
    InvalidNBT(String),
}
