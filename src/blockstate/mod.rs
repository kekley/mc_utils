use crate::borrow::nbt_string::NBTStr;

pub mod borrow;
pub mod owned;
///A `BlockState` consists of a namespace, block name, and properties
#[allow(refining_impl_trait)]
pub trait BlockStateTrait {
    fn name(&self) -> &NBTStr;

    fn iter_properties(&self) -> impl Iterator<Item = (&NBTStr, &NBTStr)>;
}
