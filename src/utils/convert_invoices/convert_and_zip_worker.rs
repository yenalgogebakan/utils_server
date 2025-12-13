use crate::utils::appstate::appstate::SharedState;
use crate::utils::common::target_types_and_formats::{
    FilenameInZipMode, TargetCompressionType, TargetType,
};
use crate::utils::common::zip_utils::ZipFile;
use crate::utils::convert_invoices::invoice_conversion_manager::{
    InvoiceConversionJob, InvoiceConversionResult, InvoiceItemForConversion,
};
use crate::utils::errors::invoice_conversion_errors::{ErrCtx, InvConvError};
use crate::utils::xslt_engine::libxslt_engine::LibXsltEngine;
use crate::utils::xslt_engine::xrust_engine::XrustEngine;
use crate::utils::xslt_engine::xslt_engine::XsltEngine;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// ---- blocking worker ----
pub fn convert_and_zip(
    request_id: &String,
    mut rx: mpsc::Receiver<InvoiceConversionJob>,
    _state: SharedState, // No use, may be required in the future
    worker_cancellation_token: CancellationToken,
    _target_type: TargetType,
    _target_compression_type: TargetCompressionType,
    filename_in_zip_mode: FilenameInZipMode,
) -> Result<InvoiceConversionResult, InvConvError> {
    let mut zip = match ZipFile::new() {
        Ok(z) => z,
        Err(e) => {
            let my_err = InvConvError::ZipFileCreationError {
                request_id: request_id.to_string(),
                source: e,
            };

            log_error(&my_err);
            return Err(my_err);
        }
    };

    let mut docs_count = 0u8;
    let mut last_processed_sira_no = 0u64;
    let mut total_html_bytes = 0u64;

    // Either use one of them
    let engine = XrustEngine::new();
    type Compiled = <XrustEngine as XsltEngine>::Compiled;

    let _engine2 = XrustEngine::new();
    type _Compiled2 = <LibXsltEngine as XsltEngine>::Compiled;

    let mut xslt_cache: HashMap<String, Compiled> = HashMap::with_capacity(4);

    // Process incoming jobs
    while let Some(invoice_conversion_job) = rx.blocking_recv() {
        if worker_cancellation_token.is_cancelled() {
            return Err(InvConvError::ClientDisconnectedError(
                "Client disconnected, task canceled".to_string(),
            ))
            .ctx("convert_and_zip:process cancelled");
        }
        let compiled_ref = match xslt_cache.entry(invoice_conversion_job.xslt_key.clone()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                // If not in cache, we MUST have data.
                let bytes = match invoice_conversion_job.xslt_data.as_ref() {
                    Some(b) => b,
                    None => {
                        let err =
                            InvConvError::XsltDataMissing(invoice_conversion_job.xslt_key.clone());
                        // Explicitly log before returning as requested
                        log_error(&err);
                        return Err(err);
                    }
                };

                let compiled = engine.compile(bytes)?;
                v.insert(compiled)
            }
        };

        match engine.transform(compiled_ref, &invoice_conversion_job.xml_data) {
            Ok(html_bytes) => {
                let filename = filename_in_zip(
                    &invoice_conversion_job.item,
                    &filename_in_zip_mode,
                    docs_count,
                );

                let current_bytes_len = html_bytes.len() as u64;

                if let Err(e) = zip.write_to_zip(&filename, html_bytes) {
                    let wrapped_err = InvConvError::ZipIOError {
                        sira_no: invoice_conversion_job.item.sira_no.unwrap_or(0).to_string(),
                        source: e,
                    };
                    log_error(&wrapped_err);
                    if wrapped_err.is_fatal() {
                        return Err(wrapped_err);
                    }

                    match zip.close_zip() {
                        Ok(bytes) => {
                            return Ok(InvoiceConversionResult {
                                data: bytes,
                                docs_count,
                                size: total_html_bytes,
                                last_processed_sira_no: Some(last_processed_sira_no),
                                request_fully_completed: false,
                            });
                        }
                        Err(zip_err) => {
                            let final_err = InvConvError::ZipError {
                                request_id: request_id.to_string(),
                                sira_no: last_processed_sira_no.to_string(),
                                source: zip_err,
                            };
                            log_error(&final_err);
                            return Err(final_err);
                        }
                    }
                }

                docs_count += 1;
                total_html_bytes += current_bytes_len;
                if let Some(sn) = invoice_conversion_job.item.sira_no {
                    last_processed_sira_no = sn;
                }
            }
            Err(e) => {
                log_error(&e);
                return Err(e);
            }
        }
    }
    match zip.close_zip() {
        Ok(bytes) => Ok(InvoiceConversionResult {
            data: bytes,
            docs_count,
            size: total_html_bytes,
            last_processed_sira_no: Some(last_processed_sira_no),
            request_fully_completed: true,
        }),
        Err(e) => {
            let my_err = InvConvError::ZipError {
                request_id: request_id.to_string(),
                sira_no: last_processed_sira_no.to_string(),
                source: e,
            };
            log_error(&my_err);
            Err(my_err)
        }
    }
}

/// Determine the filename to use inside the ZIP archive based on the specified mode.
fn filename_in_zip(
    item: &InvoiceItemForConversion,
    filename_in_zip_mode: &FilenameInZipMode,
    docs_count: u8,
) -> String {
    match filename_in_zip_mode {
        FilenameInZipMode::ExtractFromObjID => {
            // Returns an Option (Some or None)
            if let Some(filename) = item.object_id.get(8..16) {
                format!("Fat_{}", filename)
            } else {
                format!("Fat_{}", docs_count)
            }
        }
        FilenameInZipMode::IncludedInRequest => match &item.invoice_no {
            Some(value) => format!("Fat_{}", value),
            None => format!("Fat_{}", docs_count),
        },
        FilenameInZipMode::UseSiraNo => match item.sira_no {
            Some(value) => format!("Fat_{}", value),
            None => format!("Fat_{}", docs_count),
        },
        FilenameInZipMode::StartFromInvoiceOne => format!("Fat_{}", docs_count),
    }
}

fn log_error(e: &InvConvError) {
    //log::error!("Error processing sira_no {}: {:?}", sira_no, e);
}
