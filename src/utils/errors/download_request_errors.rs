use thiserror::Error;

use crate::utils::download_request::download_request_types::{DownloadFormat, DownloadType};

#[derive(Debug, Error)]
pub enum DownloadRequestError {
    #[error("can not extract database name path: {0}")]
    CanNotExtractDatabaseName(String),

    #[error("can not get connection from: {0} pool")]
    CanNotGetConnectionFromPool(#[from] bb8::RunError<bb8_tiberius::Error>),

    #[error("problem in query build and execution: {0} ")]
    CanNotExcuteQuery(#[from] tiberius::error::Error),

    #[error("invalid download type or format (downloadtype {downloadtype:?}, format {format:?})")]
    InvalidTypeFormat {
        downloadtype: DownloadType,
        format: DownloadFormat,
    },

    // Function context (preserves typed inner error)
    #[error("{func}: {source}")]
    Context {
        func: &'static str,
        #[source]
        source: Box<DownloadRequestError>,
    },
}

pub trait ErrCtx<T> {
    fn ctx(self, func: &'static str) -> Result<T, DownloadRequestError>;
}

impl<T, E> ErrCtx<T> for Result<T, E>
where
    E: Into<DownloadRequestError>,
{
    fn ctx(self, func: &'static str) -> Result<T, DownloadRequestError> {
        self.map_err(|e| DownloadRequestError::Context {
            func,
            source: Box::new(e.into()),
        })
    }
}
