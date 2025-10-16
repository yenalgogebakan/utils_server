use crate::utils::errors::object_store_errors::ObjectStoreError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FatalProcessError {
    #[error("Html conversion error: {0}")]
    HtmlConversionError(String),

    #[error(transparent)]
    ObjectStore(#[from] ObjectStoreError),

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
