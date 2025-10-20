#[derive(Debug, Clone, serde::Serialize)]
pub struct ProcessingError {
    pub invoice_id: String,
    pub error_code: Option<String>,
    pub message: String,
}

use crate::utils::errors::object_store_errors::ObjectStoreError;
use std::io;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("Ubl Not found in Object store: objectId: {0}")]
    UblNotFoundInObjectStore(String),

    #[error("Html conversion error: {0}")]
    HtmlConversionError(String),

    #[error("Found no utf char, returning untouched")]
    NonUtfCharError(#[source] std::str::Utf8Error),

    #[error(transparent)]
    ObjectStore(#[from] ObjectStoreError),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("Decompression task failed")]
    TaskJoin(#[source] JoinError),

    // Function context (preserves typed inner error)
    #[error("{func}: {source}")]
    Context {
        func: &'static str,
        #[source]
        source: Box<ProcessError>,
    },
}

pub trait ErrCtx<T> {
    fn ctx(self, func: &'static str) -> Result<T, ProcessError>;
}

impl<T, E> ErrCtx<T> for Result<T, E>
where
    E: Into<ProcessError>,
{
    fn ctx(self, func: &'static str) -> Result<T, ProcessError> {
        self.map_err(|e| ProcessError::Context {
            func,
            source: Box::new(e.into()),
        })
    }
}
