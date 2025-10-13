use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct Element<'a> {
    p: &'a PhantomData<()>,
}
