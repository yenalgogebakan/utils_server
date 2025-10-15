use crate::utils::errors::{
    db_errors::DbError, download_request_errors::DownloadRequestError,
    object_store_errors::ObjectStoreError, xml_errors::XmlError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Db(#[from] DbError),

    #[error(transparent)]
    Xml(#[from] XmlError),

    #[error(transparent)]
    DownloadRequest(#[from] DownloadRequestError),

    #[error(transparent)]
    ObjectStore(#[from] ObjectStoreError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("other: {0}")]
    Other(#[from] anyhow::Error),

    #[error("{func}: {source}")]
    Context {
        func: &'static str,
        #[source]
        source: Box<AppError>,
    },
}

// Add function-name context ergonomically at the app layer
pub trait ErrCtx<T> {
    fn ctx(self, func: &'static str) -> Result<T, AppError>;
}

impl<T, E> ErrCtx<T> for Result<T, E>
where
    E: Into<AppError>,
{
    fn ctx(self, func: &'static str) -> Result<T, AppError> {
        self.map_err(|e| AppError::Context {
            func,
            source: Box::new(e.into()),
        })
    }
}
