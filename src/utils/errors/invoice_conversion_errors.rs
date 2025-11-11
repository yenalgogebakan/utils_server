use crate::utils::errors::db_errors::DbError;
use crate::utils::errors::object_store_errors::ObjectStoreError;
use axum::http::StatusCode;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InvConvError {
    // Fatal errors - system level
    #[error("Database error: {0}")]
    DatabaseError(#[from] DbError),

    #[error("Error: {0}")]
    ServerBusyError(String),

    #[error("Error: {0}")]
    TaskJoinError(String),

    #[error("Error: {0}")]
    ClientDisconnectedError(String),

    /*
    #[error("Error: '{object_id}': {source}")]
    ObjStoreError {
        object_id: String, // To hold the ID of the object
        #[source] // Indicate that this is the underlying source error
        source: ObjectStoreError,
    },
    */
    #[error("Error: {0}")]
    ObjStoreError(#[from] ObjectStoreError),

    #[error("Zip error for  sira_no '{sira_no}': {source}")]
    ZipError {
        sira_no: String, // To hold the ID of the object
        #[source] // Indicate that this is the underlying source error
        source: zip::result::ZipError,
    },

    #[error("ZIP I/O write error for sira_no '{sira_no}': {source}")]
    ZipIOError {
        // New variant for write failures
        sira_no: String,
        #[source]
        source: io::Error, // Expected by `zip.write_all()`
    },

    #[error("Ubl Not found in Object store: object_id: {0}")]
    UblNotFoundInObjectStore(String),

    #[error("Failed to decompress XZ data for object '{object_id}': {source}")]
    DecompressError {
        object_id: String, // To hold the ID of the object
        #[source] // Indicate that this is the underlying source error
        source: io::Error,
    },

    #[error("Timeout in decompress XZ data for object '{object_id}': '{timeout_secs}'")]
    DecompressTimeout {
        object_id: String, // To hold the ID of the object
        timeout_secs: u32,
    },

    #[error("Decompression cancelled object_id: {0}")]
    DecompressCancelled(String),

    #[error("Found non utf char, returning untouched '{object_id}': {source}")]
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

    // Function context (preserves typed inner error)
    #[error("{func}: {source}")]
    Context {
        func: &'static str,
        #[source]
        source: Box<InvConvError>,
    },
}

impl InvConvError {
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            InvConvError::DatabaseError(_)
                | InvConvError::ServerBusyError(_)
                | InvConvError::TaskJoinError(_)
                | InvConvError::ClientDisconnectedError(_)
                | InvConvError::ObjStoreError { .. }
                | InvConvError::Context { .. }
        )
    }
    pub fn error_code(&self) -> i32 {
        match self {
            InvConvError::DatabaseError(_) => 1001,
            InvConvError::ServerBusyError(_) => 1002,
            InvConvError::TaskJoinError(_) => 1003,
            InvConvError::ClientDisconnectedError(_) => 1004,
            InvConvError::ObjStoreError { .. } => 1005,

            InvConvError::ZipError { .. } => 2001,
            InvConvError::ZipIOError { .. } => 2002,
            InvConvError::UblNotFoundInObjectStore(_) => 2003,
            InvConvError::DecompressError { .. } => 2004,
            InvConvError::NonUtfCharError { .. } => 2005,
            InvConvError::XMLParseError { .. } => 2006,
            InvConvError::MissingNodeError(_) => 2007,
            InvConvError::MissingTextInNodeError(_) => 2008,
            InvConvError::InvalidXsltobjectIdError(_) => 2009,
            InvConvError::DecompressTimeout { .. } => 2010,
            InvConvError::DecompressCancelled(_) => 2011,
            InvConvError::Context { source, .. } => source.error_code(),
        }
    }
    pub fn http_status(&self) -> StatusCode {
        match self {
            InvConvError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            InvConvError::ServerBusyError(_) => StatusCode::TOO_MANY_REQUESTS,
            InvConvError::TaskJoinError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            InvConvError::ClientDisconnectedError(_) => StatusCode::GATEWAY_TIMEOUT,
            InvConvError::ObjStoreError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            InvConvError::Context { source, .. } => source.http_status(),
            _ => StatusCode::OK,
        }
    }
}

pub trait ErrCtx<T> {
    fn ctx(self, func: &'static str) -> Result<T, InvConvError>;
}

impl<T, E> ErrCtx<T> for Result<T, E>
where
    E: Into<InvConvError>,
{
    fn ctx(self, func: &'static str) -> Result<T, InvConvError> {
        self.map_err(|e| InvConvError::Context {
            func,
            source: Box::new(e.into()),
        })
    }
}
