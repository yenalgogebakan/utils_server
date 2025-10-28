use crate::utils::errors::xslt_errors::{ErrCtx, XsltError};
use libxml::parser::Parser;
use libxslt::parser::parse_file;
use libxslt::stylesheet::Stylesheet;
use std::fs;
use std::path::PathBuf;

pub struct XsltStruct {
    pub xslt: Vec<u8>,
    pub xslt_path: PathBuf, // Persistent file path
    pub compiled_xslt: Option<Stylesheet>,
}
impl XsltStruct {
    pub fn new(xslt: Vec<u8>, cache_dir: &str, xslt_key: &str) -> Result<Self, XsltError> {
        // Ensure cache directory exists
        fs::create_dir_all(cache_dir).ctx("XsltStruct:create_dir")?;

        // Write XSLT to persistent cache file
        let xslt_path = PathBuf::from(cache_dir).join(format!("{}.xslt", xslt_key));
        fs::write(&xslt_path, &xslt).ctx("XsltStruct:write")?;

        Ok(Self {
            xslt,
            xslt_path,
            compiled_xslt: None,
        })
    }
    /*
    /// Compile the XSLT if not already compiled (lazy compilation)
    /// Only compiles once, then reuses
    pub fn ensure_compiled(&mut self) -> Result<(), XsltError> {
        if self.compiled_xslt.is_none() {
            let path = self
                .xslt_path
                .to_str()
                .ok_or_else(|| XsltError::XsltCompilationError("Invalid path".to_string()))?;

            let stylesheet = parse_file(path)
                .map_err(|e| XsltError::XsltCompilationError(e))
                .ctx("XsltStruct:EnsureCompiled")?;

            self.compiled_xslt = Some(stylesheet);
        }
        Ok(())
    }
    pub fn transform(&mut self, xml: &str) -> Result<String, XsltError> {
        self.transform_with_params(xml, &[])
    }
    pub fn transform_with_params(
        &mut self,
        xml: &str,
        params: &[(&str, &str)],
    ) -> Result<String, XsltError> {
        self.ensure_compiled()?;

        let parser = Parser::default();
        let xml_doc = parser
            .parse_string(xml)
            .ctx("xslt_struct:transform_wirh_params:libxml:parse")?;

        let stylesheet = self.compiled_xslt.as_mut().unwrap();
        let result_doc = stylesheet
            .transform(&xml_doc, params.to_vec())
            .map_err(|e| XsltError::XsltTransformError(e.to_string()))?;
        //.ctx("xslt_struct:transform_with_params:transform")?;

        Ok(result_doc.to_string())
    }
    */
}
