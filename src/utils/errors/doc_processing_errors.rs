use crate::utils::errors::db_errors::DbError;
use axum::http::StatusCode;
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
                | DocProcessingError::Context { .. }
        )
    }
    pub fn error_code(&self) -> i32 {
        match self {
            DocProcessingError::DatabaseError(_) => 1001,
            DocProcessingError::ServerBusyError(_) => 1002,
            DocProcessingError::TaskJoinError(_) => 1003,
            DocProcessingError::Context { source, .. } => source.error_code(),
        }
    }
    pub fn http_status(&self) -> StatusCode {
        match self {
            DocProcessingError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DocProcessingError::ServerBusyError(_) => StatusCode::SERVICE_UNAVAILABLE,
            DocProcessingError::TaskJoinError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            DocProcessingError::Context { source, .. } => source.http_status(),
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
