use libxml::parser::XmlParseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum XsltError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Can not construct tmp path of Xslt: {0}")]
    XsltTmpPathError(String),

    #[error("Can not compile xslt from tmp file: {0}")]
    XsltCompilationError(String),

    #[error("Can not transform xml file: {0}")]
    XsltTransformError(String),

    #[error("LibXml parse error:  {0}")]
    LibXmlParseError(#[from] XmlParseError),

    // Function context (preserves typed inner error)
    #[error("{func}: {source}")]
    Context {
        func: &'static str,
        #[source]
        source: Box<XsltError>,
    },
}
pub trait ErrCtx<T> {
    fn ctx(self, func: &'static str) -> Result<T, XsltError>;
}

impl<T, E> ErrCtx<T> for Result<T, E>
where
    E: Into<XsltError>,
{
    fn ctx(self, func: &'static str) -> Result<T, XsltError> {
        self.map_err(|e| XsltError::Context {
            func,
            source: Box::new(e.into()),
        })
    }
}
