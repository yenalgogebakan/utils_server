use crate::utils::errors::process_errors::ProcessError;
use roxmltree::Document;

const CBC_NS: &str = "urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2";
const LOCAL: &str = "EmbeddedDocumentBinaryObject";

pub fn extract_xslt_key_from_xml(xml: &str, object_id: &str) -> Result<String, ProcessError> {
    let doc = Document::parse(xml).map_err(|e| ProcessError::XMLParseError {
        object_id: object_id.to_string(),
        source: e,
    })?;
    let xslt_object_id = doc
        .descendants()
        .find(|n| n.has_tag_name((CBC_NS, LOCAL)))
        .and_then(|n| n.text())
        .map(|t| t.trim())
        .ok_or_else(|| ProcessError::MissingNodeError(object_id.to_string()))?;
    if xslt_object_id.is_empty() {
        return Err(ProcessError::MissingTextInNodeError(object_id.to_string()));
    }
    if !is_valid_xslt_object_id(xslt_object_id) {
        return Err(ProcessError::InvalidXsltobjectIdError(
            object_id.to_string(),
        ));
    }

    Ok("xslt key".to_string())
}

#[inline]
pub fn is_valid_xslt_object_id(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 70 {
        return false;
    }
    b[0] == b'M' && b[23] == b'=' && b[24] == b'=' && b[25] == b'S' && b[69] == b'='
}
