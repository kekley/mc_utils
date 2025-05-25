use std::str::Utf8Error;

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

impl Into<ResourceErrorKind> for std::io::Error {
    fn into(self) -> ResourceErrorKind {
        ResourceErrorKind::ErrorLoadingFile
    }
}

impl Into<ResourceErrorKind> for serde_json::Error {
    fn into(self) -> ResourceErrorKind {
        ResourceErrorKind::InvalidJSON(self)
    }
}

pub fn create_resource_error<'a, T, F: FnOnce() -> Result<T, E>, E: Into<ResourceErrorKind>>(
    path: &'a str,
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
