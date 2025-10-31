use crate::utils::errors::db_errors::DbError;
use crate::utils::errors::object_store_errors::ObjectStoreError;
use axum::http::StatusCode;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DocProcessingError {
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

    // Function context (preserves typed inner error)
    #[error("{func}: {source}")]
    Context {
        func: &'static str,
        #[source]
        source: Box<DocProcessingError>,
    },
}

impl DocProcessingError {
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            DocProcessingError::DatabaseError(_)
                | DocProcessingError::ServerBusyError(_)
                | DocProcessingError::TaskJoinError(_)
                | DocProcessingError::ClientDisconnectedError(_)
                | DocProcessingError::ObjStoreError { .. }
                | DocProcessingError::Context { .. }
        )
    }
    pub fn error_code(&self) -> i32 {
        match self {
            DocProcessingError::DatabaseError(_) => 1001,
            DocProcessingError::ServerBusyError(_) => 1002,
            DocProcessingError::TaskJoinError(_) => 1003,
            DocProcessingError::ClientDisconnectedError(_) => 1004,
            DocProcessingError::ObjStoreError { .. } => 1005,

            DocProcessingError::ZipError { .. } => 2001,
            DocProcessingError::ZipIOError { .. } => 2002,
            DocProcessingError::UblNotFoundInObjectStore(_) => 2003,
            DocProcessingError::Context { source, .. } => source.error_code(),
        }
    }
    pub fn http_status(&self) -> StatusCode {
        match self {
            DocProcessingError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DocProcessingError::ServerBusyError(_) => StatusCode::TOO_MANY_REQUESTS,
            DocProcessingError::TaskJoinError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DocProcessingError::ClientDisconnectedError(_) => StatusCode::GATEWAY_TIMEOUT,
            DocProcessingError::ObjStoreError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            DocProcessingError::Context { source, .. } => source.http_status(),
            _ => StatusCode::OK,
        }
    }
}

pub trait ErrCtx<T> {
    fn ctx(self, func: &'static str) -> Result<T, DocProcessingError>;
}

impl<T, E> ErrCtx<T> for Result<T, E>
where
    E: Into<DocProcessingError>,
{
    fn ctx(self, func: &'static str) -> Result<T, DocProcessingError> {
        self.map_err(|e| DocProcessingError::Context {
            func,
            source: Box::new(e.into()),
        })
    }
}
