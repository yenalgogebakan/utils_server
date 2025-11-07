use crate::utils::appstate::appstate::SharedState;
use crate::utils::common::comp_decompress::{DECOMPRESS_ASYNC_THRESHOLD, xz_decompress};
use crate::utils::common::download_types_and_formats::{
    DownloadFormat, DownloadType, FilenameInZipMode,
};
use crate::utils::common::san_desanitize::sanitize_fast;
use crate::utils::convert_invoices::convert_and_zip_worker::convert_and_zip;
use crate::utils::errors::invoice_conversion_errors::{ErrCtx, InvConvError};
use crate::utils::object_store::opendal_mssql_wrapper::ObjectStoreRecord;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// ----- Info about an invoice to be converted -----
#[derive(Debug, Clone)]
pub struct InvoiceItemForConversion {
    pub object_id: String,
    pub sira_no: Option<u64>,
    pub invoice_no: Option<String>,
}

/// ----- Batch of invoices to be converted -----
#[derive(Debug, Clone)]
pub struct InvoicesForConversion {
    pub target_type: DownloadType,
    pub target_format: DownloadFormat,
    pub year: String,
    pub filename_in_zip: FilenameInZipMode,

    /// Items to fetch/process
    pub items: Vec<InvoiceItemForConversion>,
}

/// ----- Success Result, data is the zipped file -----
#[derive(Debug, Clone, Default)]
pub struct InvoiceConversionResult {
    pub data: Vec<u8>,
    pub docs_count: u32,
    pub size: u64,
    pub last_processed_sira_no: Option<u64>,
    pub request_fully_completed: bool,
}

/// ----- Error Response -----
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InvoiceConversionError {
    pub error_code: i32,
    pub error_msg: String,
}
impl From<InvConvError> for InvoiceConversionError {
    fn from(error: InvConvError) -> Self {
        Self {
            error_code: error.error_code(),
            error_msg: error.to_string(), // Uses Display from thiserror
        }
    }
}

/// ----- Worker channel send struct -----
pub struct InvoiceConversionJob {
    pub item: InvoiceItemForConversion,
    pub xml_data: Vec<u8>,
    pub xslt_data: Option<Vec<u8>>,
}

pub type XsltCache = HashMap<String, Vec<u8>>;

pub async fn convert_invoices(
    state: SharedState,
    conversion_request: InvoicesForConversion,
) -> Result<InvoiceConversionResult, InvConvError> {
    let mut xlst_cache: XsltCache = HashMap::with_capacity(4);
    let filename_in_zip_mode: FilenameInZipMode = conversion_request.filename_in_zip;

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
    let token_for_worker = token.clone();
    let _cancel_on_drop = token.drop_guard();

    let state_cloned = state.clone();

    let (tx_jobs, rx_jobs) = mpsc::channel::<(InvoiceConversionJob, Vec<u8>)>(8); // Vec will hold the Zipped data

    let handle = tokio::task::spawn_blocking(move || {
        convert_and_zip(
            rx_jobs,
            state_cloned,
            permit,
            token_for_worker,
            filename_in_zip_mode,
        )
    });

    let object_store = &state.object_store;
    for (idx, item) in conversion_request.items.iter().enumerate() {
        // Check if the compressed ubl exists
        if !object_store
            .object_exists("ubls", &item.object_id, &conversion_request.year)
            .await?
        {
            return Err(InvConvError::UblNotFoundInObjectStore(item.object_id));
        }
        // get compressed ubl
        let object_store_rec: ObjectStoreRecord = object_store
            .get("ubls", &item.object_id, &conversion_request.year)
            .await?;

        let object_id_for_error_clone = item.object_id.clone();
        let decompressed = if object_store_rec.original_size >= DECOMPRESS_ASYNC_THRESHOLD {
            // Large file: offload to blocking thread
            tokio::task::spawn_blocking(move || {
                xz_decompress(
                    &object_store_rec.objcontent,
                    object_store_rec.original_size as usize,
                )
                .map_err(|e| InvConvError::DecompressError {
                    object_id: object_id_for_error_clone,
                    source: e,
                })
            })
            .await
            .map_err(|join_err| {
                InvConvError::TaskJoinError(format!(
                    "Task join error for object id: {}",
                    object_id_for_error_clone
                ))
            })
            .ctx("convert_invoices")??
        } else {
            // Small file: decompress inline
            xz_decompress(
                &object_store_rec.objcontent,
                object_store_rec.original_size as usize,
            )
            .map_err(|e| InvConvError::DecompressError {
                // Construct the error manually
                object_id: object_id_for_error_clone, // Use the object_id
                source: e,                            // The io::Error from xz_decompress
            })?
        };

        let sanitized =
            sanitize_fast(&decompressed).map_err(|e| InvConvError::NonUtfCharError {
                object_id: object_id_for_error_clone,
                source: e,
            })?;
    }
    for it in request.items.iter().cloned() {
        // async I/O (DB/object store)
        let xml = state
            .obj_store
            .get_xml_async(&it.object_id)
            .await
            .map_err(|e| DocProcessingError::Context {
                func: "get_xml_async",
                source: Box::new(e.into()),
            })?;
        if tx_jobs.send((it, xml)).await.is_err() {
            // worker gone (cancellation or fatal); stop producing
            break;
        }
        if token.is_cancelled() {
            return Err(DocProcessingError::ClientDisconnectedError(
                "cancelled".into(),
            ));
        }
    }
    drop(tx_jobs); // signal EOF

    // Placeholder implementation
    Ok(InvoiceConversionResult {
        data: Vec::new(),
        docs_count: 0,
        size: 0,
        last_processed_sira_no: None,
        request_fully_completed: true,
    })
}
