use tokio_util::bytes::Bytes;

pub trait XsltEngine {
    /// Type used to represent a compiled stylesheet for this engine.
    type Compiled;
    /// Error type returned by this engine.
    type Error;

    /// Compile an XSLT stylesheet from bytes.
    fn compile(&self, xslt: &Bytes) -> Result<Self::Compiled, Self::Error>;

    /// Transform XML using a compiled stylesheet.
    fn transform(&self, compiled: &Self::Compiled, xml: &Bytes) -> Result<Bytes, Self::Error>;
}
