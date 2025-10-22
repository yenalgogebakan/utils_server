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
use zip::result::ZipError;

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("Ubl Not found in Object store: object_id: {0}")]
    UblNotFoundInObjectStore(String),

    #[error("Failed to decompress XZ data for object '{object_id}': {source}")]
    DecompressError {
        object_id: String, // To hold the ID of the object
        #[source] // Indicate that this is the underlying source error
        source: io::Error,
    },

    #[error("Found no utf char, returning untouched '{object_id}': {source}")]
    NonUtfCharError {
        object_id: String, // To hold the ID of the object
        #[source] // Indicate that this is the underlying source error
        source: std::str::Utf8Error,
    },

    #[error("XML parse error: '{object_id}': {source}")]
    XMLParseError {
        object_id: String, // To hold the ID of the object
        #[source] // Indicate that this is the underlying source error
        source: roxmltree::Error,
    },
    #[error("EmbeddedDocumentBinaryObject not found: object_id: {0}")]
    MissingNodeError(String),

    #[error("EmbeddedDocumentBinaryObject node has no text in it: object_id: {0}")]
    MissingTextInNodeError(String),

    #[error("Xslt object id is invalid: object_id: {0}")]
    InvalidXsltobjectIdError(String),

    #[error("DocsFromObjStore : do_process : Zip error: {0}")]
    ZipError(#[from] ZipError),

    #[error("Html conversion error: {0}")]
    HtmlConversionError(String),

    #[error(transparent)]
    ObjectStoreError(#[from] ObjectStoreError),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("Tokio Spawn sync could not join handle")]
    TaskJoinError(#[source] JoinError),

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
