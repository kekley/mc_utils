use super::variant::Variant;

pub struct MultiPart {}

pub struct Case {
    when: Option<When>,
    apply: Apply,
}

pub struct Apply {
    variant: Variant,
}

pub enum When {
    OrCase(),
    AndCase(),
    SingleCase(),
}
