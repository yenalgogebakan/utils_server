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
impl Default for DownloadType {
    fn default() -> Self {
        DownloadType::Html
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum DownloadFormat {
    Zip,  // .zip
    Tzip, // .tar.xz
    Gzip, // .tar.gz  (multi-file via tar.gz)
}
impl fmt::Display for DownloadFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zip => write!(f, "Zip"),
            Self::Tzip => write!(f, "Tzip"),
            Self::Gzip => write!(f, "Gzip"),
        }
    }
}
impl Default for DownloadFormat {
    fn default() -> Self {
        DownloadFormat::Zip
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum FilenameInZipMode {
    ExtractFromObjID,
    IncludedInRequest,
    UseSiraNo,
    StartFromInvoiceOne,
}
impl fmt::Display for FilenameInZipMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExtractFromObjID => write!(f, "ExtractFromObjID"),
            Self::IncludedInRequest => write!(f, "IncludedInRequest"),
            Self::UseSiraNo => write!(f, "UseSiraNo"),
            Self::StartFromInvoiceOne => write!(f, "StartFromInvoiceOne"),
        }
    }
}
impl Default for FilenameInZipMode {
    fn default() -> Self {
        FilenameInZipMode::StartFromInvoiceOne
    }
}
