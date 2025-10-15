use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum DownloadType {
    Html,
    Pdf,
    Ubl,
    #[serde(rename = "Ubl_Xslt_Separate")]
    UblXsltSeparate,
}

impl fmt::Display for DownloadType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Html => write!(f, "Html"),
            Self::Pdf => write!(f, "Pdf"),
            Self::Ubl => write!(f, "Ubl"),
            Self::UblXsltSeparate => write!(f, "Ubl_Xslt_Separate"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum DownloadFormat {
    Zip,
    Gzip,
}
impl fmt::Display for DownloadFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zip => write!(f, "Zip"),
            Self::Gzip => write!(f, "Gzip"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, Hash, PartialEq)]
pub struct DownloadDocRequest {
    pub source_vkntckn: String,
    pub after_this: i64,
    pub download_type: DownloadType,
    pub format: DownloadFormat,
}

impl std::fmt::Display for DownloadDocRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Source vkntckn: {} after count: {} download type: {} format: {}",
            self.source_vkntckn, self.after_this, self.download_type, self.format
        )
    }
}
