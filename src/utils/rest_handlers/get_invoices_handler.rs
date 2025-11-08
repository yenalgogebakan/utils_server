use crate::utils::appstate::appstate::SharedState;
use crate::utils::common::download_types_and_formats::{
    DownloadFormat, DownloadType, FilenameInZipMode,
};
use crate::utils::convert_invoices::invoice_conversion_manager::{
    InvoiceConversionError, InvoiceConversionResult, InvoiceItemForConversion,
    InvoicesForConversion, convert_invoices,
};
use crate::utils::errors::invoice_conversion_errors::InvConvError;
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};
use tokio_util::sync::CancellationToken;

/// ----- Request Item -----
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestInvoiceItemForConversion {
    pub object_id: String,
    pub sira_no: Option<u64>,
    pub invoice_no: Option<String>,
}
impl From<RequestInvoiceItemForConversion> for InvoiceItemForConversion {
    fn from(req_item: RequestInvoiceItemForConversion) -> Self {
        InvoiceItemForConversion {
            object_id: req_item.object_id,
            sira_no: req_item.sira_no,
            invoice_no: req_item.invoice_no,
        }
    }
}
/// ----- Request -----
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestInvoicesForConversion {
    pub target_type: DownloadType,
    pub target_format: DownloadFormat,
    pub year: String,
    #[serde(default)]
    pub filename_in_zip: FilenameInZipMode,

    /// Items to fetch/process
    pub items: Vec<RequestInvoiceItemForConversion>,
}
impl From<RequestInvoicesForConversion> for InvoicesForConversion {
    fn from(req: RequestInvoicesForConversion) -> Self {
        InvoicesForConversion {
            target_type: req.target_type,
            target_format: req.target_format,
            year: req.year,
            filename_in_zip: req.filename_in_zip,
            items: req
                .items
                .into_iter()
                .map(InvoiceItemForConversion::from)
                .collect(),
        }
    }
}

/// ----- Success Response -----
#[serde_as]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResponseInvoicesForConversion {
    /// Base64-encoded ZIP bytes serialized as a string in JSON
    #[serde_as(as = "Base64")]
    pub data: Vec<u8>,

    /// Number of docs included
    pub docs_count: u32,

    /// Total byte size of the ZIP (or payload) in bytes
    pub size: u64,

    pub last_processed_sira_no: Option<u64>,
    pub request_fully_completed: bool,
}
impl From<InvoiceConversionResult> for ResponseInvoicesForConversion {
    fn from(response: InvoiceConversionResult) -> Self {
        ResponseInvoicesForConversion {
            data: response.data,
            docs_count: response.docs_count,
            size: response.size,
            last_processed_sira_no: response.last_processed_sira_no,
            request_fully_completed: response.request_fully_completed,
        }
    }
}

/// ----- Error Response -----
impl IntoResponse for InvConvError {
    fn into_response(self) -> Response {
        let status = self.http_status();
        let body = Json(InvoiceConversionError::from(self));
        (status, body).into_response()
    }
}

pub async fn get_invoices_handler(
    State(state): State<SharedState>,
    Json(request): Json<RequestInvoicesForConversion>,
) -> Result<(StatusCode, Json<ResponseInvoicesForConversion>), InvConvError> {
    // Try to acquire without waiting; fail fast if saturated.
    let permit = state
        .blocking_limiter
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            InvConvError::ServerBusyError(
                "Server is handling the maximum number of heavy tasks. Please retry.".to_string(),
            )
        })?;

    let token = CancellationToken::new();
    let _cancel_on_drop = token.clone().drop_guard(); // guard borrows a clone
    let cancellation_token = token; // move the original downstream

    let request: InvoicesForConversion = request.into(); // convert
    let invoice_conversion_result: InvoiceConversionResult =
        convert_invoices(state.clone(), request, permit, cancellation_token).await?;

    // Here means no fatal error
    match invoice_conversion_result.request_fully_completed {
        true => Ok((StatusCode::OK, Json(invoice_conversion_result.into()))),
        false => Ok((
            StatusCode::PARTIAL_CONTENT,
            Json(invoice_conversion_result.into()),
        )),
    }

    /*
        let response = handle
                .await
                .map_err(|join_err| {
                    if join_err.is_panic() {
                        //log::error!("Blocking task panicked: {:?}", join_err);
                        InvConvError::TaskJoinError(
                            "Internal error: task panicked during processing".to_string()
                        )
                    } else if join_err.is_cancelled() {
                        //log::warn!("Blocking task was cancelled");
                        InvConvError::TaskJoinError(
                            "Task was cancelled".to_string()
                        )
                    } else {
                        //log::error!("Blocking task failed: {:?}", join_err);
                        InvConvError::TaskJoinError(
                            format!("Task execution failed: {}", join_err)
                        )
                    }
                })?  // First ? handles JoinError
                ?;
    */
}
