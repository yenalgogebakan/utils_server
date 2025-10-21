use crate::utils::errors::process_errors::ProcessError;
use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;

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
pub fn sanitize_fast<'a>(content: &'a [u8]) -> Result<Cow<'a, [u8]>, ProcessError> {
    const NO_SANITIZATION: bool = false;
    if NO_SANITIZATION {
        return Ok(Cow::Borrowed(content));
    }

    let s = std::str::from_utf8(content).map_err(|e| ProcessError::NonUtfCharError(e))?;

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
        Cow::Borrowed(_) => Ok(Cow::Borrowed(content)), // No replacements: zero-copy!
        Cow::Owned(s) => Ok(Cow::Owned(s.into_bytes())), // Replacements made: owned data
    }
}
