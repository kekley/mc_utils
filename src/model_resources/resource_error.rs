pub struct ResourceError {
    pub file: String,
    pub kind: ResourceErrorKind,
}

#[derive(Debug)]
pub enum ResourceErrorKind {
    ErrorLoadingFile,
    InvalidJSON(serde_json::Error),
    MissingField(String),
    InvalidField(String),
}

impl From<std::io::Error> for ResourceErrorKind {
    fn from(val: std::io::Error) -> Self {
        ResourceErrorKind::ErrorLoadingFile
    }
}

impl From<serde_json::Error> for ResourceErrorKind {
    fn from(val: serde_json::Error) -> Self {
        ResourceErrorKind::InvalidJSON(val)
    }
}

pub fn create_resource_error<T, F: FnOnce() -> Result<T, E>, E: Into<ResourceErrorKind>>(
    path: &str,
    f: F,
) -> Result<T, ResourceError> {
    match f() {
        Ok(t) => Ok(t),
        Err(err) => Err(ResourceError {
            file: path.to_string(),
            kind: err.into(),
        }),
    }
}
