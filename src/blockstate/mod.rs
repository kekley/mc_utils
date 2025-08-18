pub mod borrow;
pub mod owned;

pub trait BlockStateTrait<'a> {
    type StringType: 'a;

    fn name(&self) -> Self::StringType;

    fn iter_properties(&self) -> impl Iterator<Item = (Self::StringType, Self::StringType)>;
}
