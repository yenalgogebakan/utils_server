use crate::utils::appstate::appstate;
use crate::utils::common::download_types_and_formats::{
    DownloadFormat, DownloadType, FilenameInZipMode,
};
use crate::utils::docs_from_objstore::docs_from_objstore::DocsFromObjStore;
use crate::utils::object_store::object_store::Store;
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};

/// ----- Request -----
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsFromObjStoreReq {
    pub target_type: DownloadType,
    pub target_format: DownloadFormat,
    pub year: String,
    #[serde(default)]
    pub filename_in_zip: FilenameInZipMode,

    /// Items to fetch/process
    pub items: Vec<DocRequestItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocRequestItem {
    pub object_id: String,
    pub sira_no: Option<i64>,
    pub invoice_no: Option<String>,
}

/// ----- Success Response -----

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsFromObjStoreResponse {
    /// Base64-encoded ZIP bytes serialized as a string in JSON
    #[serde_as(as = "Base64")]
    pub data: Vec<u8>,

    /// Number of docs included
    pub docs_count: u32,

    /// Total byte size of the ZIP (or payload) in bytes
    pub size: u64,

    pub last_processed_sira_no: Option<i64>,
}

/// ----- Error Response -----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsFromObjStoreErrorResponse {
    pub error: i32,
    pub error_text: String,
    pub error_msg: String,
}

pub async fn docs_from_objstore_handler(
    State(state): State<appstate::SharedState>,
    Json(request): Json<DocsFromObjStoreReq>,
) -> Result<
    (StatusCode, Json<DocsFromObjStoreResponse>),
    (StatusCode, Json<DocsFromObjStoreErrorResponse>),
> {
    let process: DocsFromObjStore = request.into();

    return Ok((
        StatusCode::OK,
        Json(DocsFromObjStoreResponse {
            data: Vec::new(),
            docs_count: 0,
            size: 0,
            last_processed_sira_no: None,
        }),
    ));
}
