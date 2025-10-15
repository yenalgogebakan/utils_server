use thiserror::Error;

#[derive(Debug, Error)]
pub enum XmlError {
    #[error("parse error: {0}")]
    Parse(String),

    #[error("validation failed: {0}")]
    Validation(String),
}
