use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    // Pool builder failure:
    #[error("bb8 pool build failed: {0}")]
    PoolBuild(#[from] bb8::RunError<bb8_tiberius::Error>),

    // Getting a connection, manager-level issues:
    #[error("tiberius manager error: {0}")]
    Bb8Tiberius(#[from] bb8_tiberius::Error),

    // Direct Tiberius operation errors:
    #[error("tiberius error: {0}")]
    Tiberius(#[from] tiberius::error::Error),

    #[error("invalid config: {0}")]
    Config(&'static str),

    // 💥 Custom domain-specific variant
    #[error("wrong database name: expected '{expected}', got '{found}'")]
    WrongDatabaseName {
        expected: &'static str,
        found: String,
    },

    // Function context (preserves typed inner error)
    #[error("{func}: {source}")]
    Context {
        func: &'static str,
        #[source]
        source: Box<DbError>,
    },
}

// Add function-name context ergonomically inside the DB layer
pub trait ErrCtx<T> {
    fn ctx(self, func: &'static str) -> Result<T, DbError>;
}

impl<T, E> ErrCtx<T> for Result<T, E>
where
    E: Into<DbError>,
{
    fn ctx(self, func: &'static str) -> Result<T, DbError> {
        self.map_err(|e| DbError::Context {
            func,
            source: Box::new(e.into()),
        })
    }
}
