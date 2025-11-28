use std::str;
use tokio_util::bytes::Bytes;

// xrust imports – adjust paths to match your version
use xrust::Node;
use xrust::SequenceTrait;
use xrust::item::Item;
use xrust::parser::xml::parse;
use xrust::transform::context::{Context, StaticContextBuilder};
use xrust::trees::smite::RNode;
use xrust::xdmerror::{Error as XrustError, ErrorKind};
use xrust::xslt::from_document;

use crate::utils::errors::invoice_conversion_errors::{ErrCtx, InvConvError};
use crate::utils::xslt_engine::xslt_engine::XsltEngine;

/// What we store in the cache: compiled stylesheet context.
pub type XrustCompiledStylesheet = Context<RNode>;

pub struct XrustEngine;

impl XrustEngine {
    pub fn new() -> Self {
        XrustEngine
    }

    /// Helper: parse a string into an RNode document.
    fn parse_xml(s: &str) -> Result<RNode, XrustError> {
        let doc = RNode::new_document();
        parse(doc.clone(), s, None)?; // fills the document
        Ok(doc)
    }
}

impl XsltEngine for XrustEngine {
    type Compiled = XrustCompiledStylesheet;
    type Error = InvConvError;

    fn compile(&self, xslt: &Bytes) -> Result<Self::Compiled, Self::Error> {
        // ZERO-COPY: Bytes -> &str (UTF-8 check only, no memcopy).
        let xslt_str = str::from_utf8(xslt)
            .map_err(|e| InvConvError::XRustXsltError(e.to_string()))
            .ctx("XrustEngine:compile:xslt_str")?;

        // Parse stylesheet XML into a document tree
        let style_doc = Self::parse_xml(xslt_str)
            .map_err(|e| InvConvError::XRustXsltError((e.to_string())))
            .ctx("XrustEngine:compile:style_doc")?;

        // Compile stylesheet into a Context<RNode>.
        // from_document will resolve xsl:include/import using the parser you give it.
        let ctx = from_document(
            style_doc,
            None,                   // base URI
            |s| Self::parse_xml(s), // parser for included stylesheets
            |_| Ok(String::new()),  // loader for external resources (if needed)
        )
        .map_err(|e| InvConvError::XRustXsltError(e.to_string()))
        .ctx("XrustEngine:compile:ctx from_document")?;

        Ok(ctx)
    }

    fn transform(&self, compiled: &Self::Compiled, xml: &Bytes) -> Result<Bytes, Self::Error> {
        // ZERO-COPY: Bytes -> &str
        let xml_str = str::from_utf8(xml)
            .map_err(|e| InvConvError::XRustXsltError(e.to_string()))
            .ctx("XrustEngine:transform:xml_str")?;

        // Parse the source XML into a document
        let src_doc = Self::parse_xml(xml_str)
            .map_err(|e| InvConvError::XRustXsltError(e.to_string()))
            .ctx("XrustEngine:transform:src_doc")?;
        let src_item = Item::Node(src_doc);

        // Clone the context (cheap; internally Rc-based).
        let mut ctx = compiled.clone();

        // Set the context item and result document
        ctx.context(vec![src_item], 0);
        ctx.result_document(RNode::new_document());

        // Build static context (message handler, URI resolver, etc.)
        let mut static_context = StaticContextBuilder::new()
            .message(|_| Ok(())) // ignore xsl:message
            .fetcher(|_| {
                Err(XrustError::new(
                    ErrorKind::NotImplemented,
                    "document() and external fetcher not implemented",
                ))
            })
            .parser(|_| {
                Err(XrustError::new(
                    ErrorKind::NotImplemented,
                    "external parser not implemented",
                ))
            })
            .build();

        // Evaluate transformation
        let result_seq = ctx
            .evaluate(&mut static_context)
            .map_err(|e| InvConvError::XRustXsltError(e.to_string()))
            .ctx("XrustEngine:transform:result_seq")?;

        // Serialize result to XML/HTML string. This allocates once.
        let out = result_seq.to_xml();

        // Wrap in Bytes – one copy for the final buffer.
        Ok(Bytes::from(out))
    }
}
