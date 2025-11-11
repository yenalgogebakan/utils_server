use crate::utils::errors::invoice_conversion_errors::{ErrCtx, InvConvError};
use roxmltree::Document;
use tokio_util::bytes;

const CBC_NS: &str = "urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2";
const LOCAL: &str = "EmbeddedDocumentBinaryObject";

pub fn extract_xslt_key_from_xml(
    xml_bytes_owned: bytes::Bytes,
    object_id: &str,
) -> Result<String, InvConvError> {
    let xml_str = std::str::from_utf8(xml_bytes_owned.as_ref())
        .map_err(|e| InvConvError::NonUtfCharError {
            object_id: object_id.to_string(),
            source: e,
        })
        .ctx("extract_xslt_key_from_xml")?;

    let doc = Document::parse(xml_str)
        .map_err(|e| InvConvError::XMLParseError {
            object_id: object_id.to_string(),
            source: e,
        })
        .ctx("extract_xslt_key_from_xml")?;

    // 1) Find the node; the borrow of the iterator ends right here.
    let node = doc
        .descendants()
        .find(|n| n.has_tag_name((CBC_NS, LOCAL)))
        .ok_or_else(|| InvConvError::MissingNodeError(object_id.to_string()))
        .ctx("extract_xslt_key_from_xml")?;
    // 2) Get text (borrows from `xml` via `doc`'s lifetime parameter).
    let text = node
        .text()
        .ok_or_else(|| InvConvError::MissingTextInNodeError(object_id.to_string()))
        .ctx("extract_xslt_key_from_xml")?;
    // 3) Trim returns a subslice; still tied to `xml`.
    let trimmed: &str = text.trim();

    if trimmed.is_empty() {
        return Err(InvConvError::MissingTextInNodeError(object_id.to_string()))
            .ctx("extract_xslt_key_from_xml");
    }

    if !is_valid_xslt_object_id(trimmed) {
        return Err(InvConvError::InvalidXsltobjectIdError(
            object_id.to_string(),
        ));
    }

    Ok(trimmed.to_string())
}

#[inline]
pub fn is_valid_xslt_object_id(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 70 {
        return false;
    }
    b[0] == b'M' && b[23] == b'=' && b[24] == b'=' && b[25] == b'S' && b[69] == b'='
}
