use crate::utils::appstate::appstate::SharedState;
use crate::utils::common::download_types_and_formats::{
    DownloadFormat, DownloadType, FilenameInZipMode,
};
use crate::utils::docs_from_objstore::process_docs_from_objstore_spawn::process_docs_from_objstore_spawn;
use crate::utils::errors::doc_processing_errors::DocProcessingError;
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocsFromObjStoreResponse {
    /// Base64-encoded ZIP bytes serialized as a string in JSON
    #[serde_as(as = "Base64")]
    pub data: Vec<u8>,

    /// Number of docs included
    pub docs_count: u32,

    /// Total byte size of the ZIP (or payload) in bytes
    pub size: u64,

    pub last_processed_sira_no: Option<i64>,
    pub request_fully_completed: bool,
}

/// ----- Error Response -----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsFromObjStoreErrorResponse {
    pub error_code: i32,
    pub error_msg: String,
}
impl From<DocProcessingError> for DocsFromObjStoreErrorResponse {
    fn from(error: DocProcessingError) -> Self {
        Self {
            error_code: error.error_code(),
            error_msg: error.to_string(), // Uses Display from thiserror
        }
    }
}
impl IntoResponse for DocProcessingError {
    fn into_response(self) -> Response {
        let status = self.http_status();
        let body = Json(DocsFromObjStoreErrorResponse::from(self));
        (status, body).into_response()
    }
}

pub async fn docs_from_objstore_spawn_handler(
    State(state): State<SharedState>,
    Json(request): Json<DocsFromObjStoreReq>,
) -> Result<(StatusCode, Json<DocsFromObjStoreResponse>), DocProcessingError> {
    // Try to acquire without waiting; fail fast if saturated.
    let permit = state
        .blocking_limiter
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            DocProcessingError::ServerBusyError(
                "Server is handling the maximum number of heavy tasks. Please retry.".to_string(),
            )
        })?;

    let state_cloned = state.clone();
    let req_owned = request; // move
    let handle = tokio::task::spawn_blocking(move || {
        process_docs_from_objstore_spawn(req_owned, state_cloned, permit)
    });

    let response = handle
            .await
            .map_err(|join_err| {
                if join_err.is_panic() {
                    //log::error!("Blocking task panicked: {:?}", join_err);
                    DocProcessingError::TaskJoinError(
                        "Internal error: task panicked during processing".to_string()
                    )
                } else if join_err.is_cancelled() {
                    //log::warn!("Blocking task was cancelled");
                    DocProcessingError::TaskJoinError(
                        "Task was cancelled".to_string()
                    )
                } else {
                    //log::error!("Blocking task failed: {:?}", join_err);
                    DocProcessingError::TaskJoinError(
                        format!("Task execution failed: {}", join_err)
                    )
                }
            })?  // First ? handles JoinError
            ?;

    Ok((StatusCode::OK, Json(response)))
}
