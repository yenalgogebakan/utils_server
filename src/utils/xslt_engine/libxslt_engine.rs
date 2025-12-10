use crate::utils::errors::invoice_conversion_errors::{ErrCtx, InvConvError};
use crate::utils::xslt_engine::xslt_engine::XsltEngine;
use libxml::parser::Parser;
use libxslt::parser::parse_file;
use libxslt::stylesheet::Stylesheet;
use tokio_util::bytes::Bytes;

pub struct LibXsltEngine;

impl LibXsltEngine {
    pub fn new() -> Self {
        LibXsltEngine
    }
}

impl XsltEngine for LibXsltEngine {
    type Compiled = Stylesheet;
    type Error = InvConvError;

    fn compile(&self, _xslt: &Bytes) -> Result<Self::Compiled, Self::Error> {
        Err(InvConvError::XRustXsltError(
            "LibXsltEngine: compile not implemented".to_string(),
        ))
    }

    fn transform(&self, _compiled: &Self::Compiled, xml: &Bytes) -> Result<Bytes, Self::Error> {
        Err(InvConvError::XRustXsltError(
            "LibXsltEngine: compile not implemented".to_string(),
        ))
    }
}
