pub mod borrow;
pub mod owned;

pub trait BlockStateTrait {
    type StringType;

    fn name(&self) -> Self::StringType;

    fn iter_properties(&self) -> impl Iterator<Item = (&Self::StringType, &Self::StringType)>;
}
