use thiserror::Error;

#[derive(Debug, Error)]
pub enum FatalProcessError {
    #[error("can not extract database name path: {0}")]
    CanNotExtractDatabaseName(String),

    #[error("bb8 pool Error: {0}")]
    PoolBuild(#[from] bb8::RunError<bb8_tiberius::Error>),

    #[error("OpenDAL error: {0}")]
    OpenDALError(#[from] opendal::Error),

    #[error("MsSqlStore error: {0}")]
    MsSqlStoreError(#[from] tiberius::error::Error),

    #[error("Multiple records found in bucket '{0}' for key '{1}'")]
    MultipleRecordsFound(String, String),

    #[error("Missing field '{0}' ")]
    MissingField(String),

    #[error("{0}")]
    Other(#[from] anyhow::Error),

    // Function context (preserves typed inner error)
    #[error("{func}: {source}")]
    Context {
        func: &'static str,
        #[source]
        source: Box<FatalProcessError>,
    },
}

pub trait ErrCtx<T> {
    fn ctx(self, func: &'static str) -> Result<T, FatalProcessError>;
}

impl<T, E> ErrCtx<T> for Result<T, E>
where
    E: Into<FatalProcessError>,
{
    fn ctx(self, func: &'static str) -> Result<T, FatalProcessError> {
        self.map_err(|e| FatalProcessError::Context {
            func,
            source: Box::new(e.into()),
        })
    }
}
