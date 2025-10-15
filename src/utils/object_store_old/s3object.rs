use crate::utils::cert::cert::Cert;
use crate::utils::xslt::xslt::Xslt;
use std::fmt;

#[derive(Default, Debug, serde::Serialize)]
pub struct S3Object {
    pub year: i32,
    pub path: String,
    pub path_sanitized: String,
    pub bucket: String,
    pub only_ubl: Vec<u8>,
    pub ubl: Vec<u8>,
    pub full_ubl: Vec<u8>,

    pub xslt: Option<Box<Xslt>>,
    pub cert: Option<Box<Cert>>,
}

impl fmt::Display for S3Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Year: if 0, show 0
        let year_str = if self.year == 0 {
            "0".to_string()
        } else {
            self.year.to_string()
        };

        // Path: if empty, show "unset"
        let path_str = if self.path.is_empty() {
            "unset".to_string()
        } else {
            self.path.clone()
        };

        let path_sanitized_str = if self.path_sanitized.is_empty() {
            "unset".to_string()
        } else {
            self.path_sanitized.clone()
        };

        // Vec<u8>: print nothing if empty, or maybe print length
        let only_ubl_len_str = if self.only_ubl.is_empty() {
            "0".to_string()
        } else {
            format!("[{} bytes]", self.only_ubl.len())
        };

        write!(
            f,
            "S3Object {{ year: {}, path: {}, sanizied path: {} only_ubl_len: {} }}",
            year_str, path_str, path_sanitized_str, only_ubl_len_str
        )
    }
}
