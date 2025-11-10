use crate::utils::appstate::appstate::SharedState;
use crate::utils::common::comp_decompress::{DECOMPRESS_ASYNC_THRESHOLD, xz_decompress};
use crate::utils::common::download_types_and_formats::{
    DownloadFormat, DownloadType, FilenameInZipMode,
};
use crate::utils::common::san_desanitize::sanitize_fast;
use crate::utils::convert_invoices::convert_and_zip_worker::convert_and_zip;
use crate::utils::convert_invoices::extract_xslt_key_from_xml::extract_xslt_key_from_xml;
use crate::utils::errors::invoice_conversion_errors::{ErrCtx, InvConvError};
use crate::utils::errors::log_error::log_error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
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

/// Work item sent from the async producer to the blocking worker.
///
/// - `xml_data` must be the **sanitized** XML bytes to transform.
/// - `xslt_key` identifies the stylesheet for caching/reuse on the worker.
/// - `xslt_data` is **Some** only the first time a given `xslt_key` appears,
///   carrying the stylesheet bytes; subsequent jobs with the same key set it to `None`.
pub struct InvoiceConversionJob {
    pub item: InvoiceItemForConversion, // owned clone; channel requires 'static
    pub xml_data: Vec<u8>,              // sanitized XML bytes
    pub xslt_key: String,               // object-store key / identifier
    pub xslt_data: Option<Arc<[u8]>>,   // stylesheet bytes (first send only)
}

pub type XsltCache = HashMap<String, Arc<[u8]>>;

pub async fn convert_invoices(
    state: SharedState,
    conversion_request: InvoicesForConversion,
    _permit: tokio::sync::OwnedSemaphorePermit,
    cancellation_token: CancellationToken,
) -> Result<InvoiceConversionResult, InvConvError> {
    let mut xlst_cache: XsltCache = HashMap::with_capacity(4);
    let filename_in_zip_mode: FilenameInZipMode = conversion_request.filename_in_zip;

    let (tx_jobs, rx_jobs) = mpsc::channel::<InvoiceConversionJob>(8); // Vec will hold the Zipped data

    let state_cloned = state.clone();

    let worker_token = CancellationToken::new();
    let worker_cancellation_token = worker_token.clone(); // move the original downstream
    let handle = tokio::task::spawn_blocking(move || {
        convert_and_zip(rx_jobs, state_cloned, worker_token, filename_in_zip_mode)
    });

    let object_store = &state.object_store;
    for (idx, item) in conversion_request.items.iter().enumerate() {
        if cancellation_token.is_cancelled() {
            worker_cancellation_token.cancel();
            drop(tx_jobs);
            return Err(InvConvError::ClientDisconnectedError(
                "Client disconnected, task canceled".to_string(),
            ))
            .ctx("convert_invoices:process cancelled");
        }
        // Check if the compressed ubl exists
        if !object_store
            .object_exists("ubls", &item.object_id, &conversion_request.year)
            .await?
        {
            let err = InvConvError::UblNotFoundInObjectStore(item.object_id.clone());
            log_error(&err);
            // stop the pipeline
            worker_cancellation_token.cancel();
            drop(tx_jobs);

            let worker_res = handle
                .await
                .map_err(|e| InvConvError::TaskJoinError(e.to_string()))?;

            if err.is_fatal() {
                return Err(err).ctx("convert_invoices");
            } else {
                return worker_res;
            }
        }
        // get compressed ubl
        let object_store_rec = match object_store
            .get("ubls", &item.object_id, &conversion_request.year)
            .await
        {
            Ok(rec) => rec, // <— bind and continue below
            Err(err) => {
                let inv_err: InvConvError = err.into();
                log_error(&inv_err);

                // stop the pipeline
                worker_cancellation_token.cancel();
                drop(tx_jobs);

                // wait worker to finalize/stop
                let worker_res = handle
                    .await
                    .map_err(|e| InvConvError::TaskJoinError(e.to_string()))?;

                if inv_err.is_fatal() {
                    return Err(inv_err).ctx("convert_invoices"); // no body
                } else {
                    return worker_res; // partial body from worker
                }
            }
        };
        /*
        if cancellation_token.is_cancelled() {
            worker_cancellation_token.cancel();
            drop(tx_jobs);
            return Err(InvConvError::ClientDisconnectedError(
                "Client disconnected, task canceled".to_string(),
            ))
            .ctx("convert_invoices:process cancelled");
        }
        */
        let original_size = object_store_rec.original_size as usize;
        let decompressed: Vec<u8> = if object_store_rec.original_size >= DECOMPRESS_ASYNC_THRESHOLD
        {
            // Large file: offload to blocking thread
            let obj_bytes = object_store_rec.objcontent;
            let object_id_clone = item.object_id.clone();

            // JoinHandle<Result<Vec<u8>, InvConvError>>
            let join_out = tokio::task::spawn_blocking(move || {
                xz_decompress(&obj_bytes, original_size).map_err(|e| {
                    InvConvError::DecompressError {
                        object_id: object_id_clone,
                        source: e,
                    }
                })
            })
            .await
            .map_err(|e| InvConvError::TaskJoinError(e.to_string()))
            .ctx("convert_invoices")?;

            match join_out {
                Ok(bytes) => bytes, // happy path
                Err(inv_err) => {
                    // ❌ DecompressError from worker = NON-FATAL → stop pipeline and return partial
                    // (If some other InvConvError variant could appear here and should be fatal,
                    //  you can branch on it; for now treat all here as non-fatal as per your rule.)
                    log_error(&inv_err);
                    worker_cancellation_token.cancel();
                    drop(tx_jobs);

                    let worker_res = handle
                        .await
                        .map_err(|e| InvConvError::TaskJoinError(e.to_string()))?;
                    return worker_res; // partial result (request_fully_completed = false)
                }
            }
        } else {
            // SMALL: inline decompress; classify errors as NON-FATAL
            match xz_decompress(&object_store_rec.objcontent, original_size) {
                Ok(bytes) => bytes,
                Err(e) => {
                    let inv_err = InvConvError::DecompressError {
                        object_id: item.object_id.clone(),
                        source: e,
                    };
                    log_error(&inv_err);

                    // stop pipeline and return partial
                    worker_cancellation_token.cancel();
                    drop(tx_jobs);

                    let worker_res = handle
                        .await
                        .map_err(|e| InvConvError::TaskJoinError(e.to_string()))?;
                    return worker_res; // partial result
                }
            }
        };

        let sanitized: std::borrow::Cow<'_, [u8]> = match sanitize_fast(&decompressed) {
            Ok(s) => s,
            Err(e) => {
                let inv_err = InvConvError::NonUtfCharError {
                    object_id: item.object_id.clone(),
                    source: e,
                };
                log_error(&inv_err);

                // stop pipeline and return partial
                worker_cancellation_token.cancel();
                drop(tx_jobs);

                let worker_res = handle
                    .await
                    .map_err(|e| InvConvError::TaskJoinError(e.to_string()))?;
                return worker_res; // partial result
            }
        };
        // This does NOT copy; it just validates UTF-8 once.
        /*
        let sanitized_str: &str = match std::str::from_utf8(&sanitized) {
            Ok(s) => s,
            Err(e) => {
                // If sanitize_fast guarantees UTF-8, you can `unreachable!()` here;
                // otherwise treat as the same non-fatal path:
                let inv_err = InvConvError::NonUtfCharError {
                    object_id: item.object_id.clone(),
                    source: e.into(),
                };
                log_error(&inv_err);
                worker_cancellation_token.cancel();
                drop(tx_jobs);
                let worker_res = handle
                    .await
                    .map_err(|je| InvConvError::TaskJoinError(je.to_string()))?;
                return worker_res;
            }
        };
        */

        //extract xslt key
        let xslt_key: String = match extract_xslt_key_from_xml(sanitized, &item.object_id) {
            Ok(k) => k,
            Err(e) => {
                log_error(&e);

                // stop the pipeline
                worker_cancellation_token.cancel();
                drop(tx_jobs);

                // wait worker to finalize/stop
                let worker_res = handle
                    .await
                    .map_err(|e| InvConvError::TaskJoinError(e.to_string()))?;

                if e.is_fatal() {
                    return Err(e).ctx("convert_invoices"); // no body
                } else {
                    return worker_res; // partial body from worker
                }
            }
        };

        // get xslt from cache or objstore
    }

    // Placeholder implementation
    Ok(InvoiceConversionResult {
        data: Vec::new(),
        docs_count: 0,
        size: 0,
        last_processed_sira_no: None,
        request_fully_completed: true,
    })
}
