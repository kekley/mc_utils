use crate::chunk_new::borrow::Chunk;

pub struct Region<'a> {
    header: &'a [u8],
}

impl<'a> Region<'a> {
    pub fn get_chunk(&self)->Chunk<'a, {}
}
