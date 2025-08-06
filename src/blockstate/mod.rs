pub mod borrow;
pub mod owned;

pub trait BlockStateTrait {
    type StringType;
    type PropertiesIter;

    fn name(&self) -> Self::StringType;

    fn iter_properties(&self) -> Self::PropertiesIter;
}
