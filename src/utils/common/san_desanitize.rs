use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;
use tokio_util::bytes;

static ENTITY_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"&#((x[0-9A-Fa-f]+|\d+));").unwrap());
#[inline]
fn is_xml_char(code: u32) -> bool {
    code == 0x9
        || code == 0xA
        || code == 0xD
        || (0x20..0xD800).contains(&code)
        || (0xE000..0xFFFE).contains(&code)
        || (0x10000..0x110000).contains(&code)
}

/// - Looks for numeric entities like &#x1F; or &#31;
/// - Replaces only those *invalid for XML* with "-sanitized-$2--"
/// - Returns input on errors (matching your try/catch)
pub fn sanitize_fast(content_bytes: bytes::Bytes) -> Result<bytes::Bytes, std::str::Utf8Error> {
    const NO_SANITIZATION: bool = false;

    let s = std::str::from_utf8(content_bytes.as_ref())?;

    if NO_SANITIZATION {
        return Ok(content_bytes);
    }

    // Replace invalid entities
    let replaced = ENTITY_RE.replace_all(s, |caps: &regex::Captures| {
        let g2 = &caps[1]; // e.g. "x1F" or "31"

        let code_opt = if let Some(hex) = g2.strip_prefix(|c| c == 'x' || c == 'X') {
            u32::from_str_radix(hex, 16).ok()
        } else {
            g2.parse::<u32>().ok()
        };

        match code_opt {
            Some(code) if !is_xml_char(code) => format!("-sanitized-{}--", g2),
            _ => caps.get(0).unwrap().as_str().to_string(),
        }
    });

    // Return based on whether replacements were made
    match replaced {
        Cow::Borrowed(_) => Ok(content_bytes), // No replacements: zero-copy!
        Cow::Owned(s) => Ok(bytes::Bytes::from(s.into_bytes())), // Replacements made: owned data
    }
}
