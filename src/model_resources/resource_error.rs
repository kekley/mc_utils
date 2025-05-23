pub struct ResourceError {
    pub file: String,
    pub kind: ResourceErrorKind,
}

pub enum ResourceErrorKind {
    FileNotFound,
    InvalidJSON(serde_json::Error),
    MissingField(String),
    InvalidField(String),
}
