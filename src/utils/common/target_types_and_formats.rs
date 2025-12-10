use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum TargetType {
    Html,
    Pdf,
    Ubl,
    #[serde(rename = "Ubl_Xslt_Separate")]
    UblXsltSeparate,
}
impl fmt::Display for TargetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Html => write!(f, "Html"),
            Self::Pdf => write!(f, "Pdf"),
            Self::Ubl => write!(f, "Ubl"),
            Self::UblXsltSeparate => write!(f, "Ubl_Xslt_Separate"),
        }
    }
}
impl Default for TargetType {
    fn default() -> Self {
        TargetType::Html
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum TargetCompressionType {
    Zip,  // .zip
    Tzip, // .tar.xz
    Gzip, // .tar.gz  (multi-file via tar.gz)
}
impl fmt::Display for TargetCompressionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zip => write!(f, "Zip"),
            Self::Tzip => write!(f, "Tzip"),
            Self::Gzip => write!(f, "Gzip"),
        }
    }
}
impl Default for TargetCompressionType {
    fn default() -> Self {
        TargetCompressionType::Zip
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub enum FilenameInZipMode {
    ExtractFromObjID,
    UseSiraNo,
    IncludedInRequest,
    #[default]
    StartFromInvoiceOne,
}
impl fmt::Display for FilenameInZipMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExtractFromObjID => write!(f, "ExtractFromObjID"),
            Self::UseSiraNo => write!(f, "UseSiraNo"),
            Self::IncludedInRequest => write!(f, "IncludedInRequest"),
            Self::StartFromInvoiceOne => write!(f, "StartFromInvoiceOne"),
        }
    }
}
