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
use std::io::{Read, Seek, SeekFrom, Write};
use tempfile::tempfile;
use tokio::sync::mpsc;
//use tokio_util::bytes;
use tokio_util::sync::CancellationToken;
use zip::{CompressionMethod, ZipWriter, write::FileOptions};

/// ---- blocking worker ----
pub fn convert_and_zip(
    request_id: &String,
    mut rx: mpsc::Receiver<InvoiceConversionJob>,
    _state: SharedState,
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
            if my_err.is_fatal() {
                return Err(my_err);
            }
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
    type Compiled2 = <LibXsltEngine as XsltEngine>::Compiled;

    let mut xslt_cache: HashMap<String, Compiled> = HashMap::with_capacity(4);

    // Process incoming jobs
    while let Some(invoice_conversion_job) = rx.blocking_recv() {
        if worker_cancellation_token.is_cancelled() {
            return Err(InvConvError::ClientDisconnectedError(
                "Client disconnected, task canceled".to_string(),
            ))
            .ctx("convert_and_zip:process cancelled");
        }

        let compiled = if let Some(cached) = xslt_cache.get(&invoice_conversion_job.xslt_key) {
            cached
        } else {
            // not in cache, xslt data should be present
            let bytes = invoice_conversion_job
                .xslt_data
                .as_ref()
                .ok_or(InvConvError::XsltDataMissing(
                    invoice_conversion_job.xslt_key.clone(),
                ))
                .ctx("convert_and_zip:xslt_data missing")?;

            let compiled = engine.compile(bytes)?;

            xslt_cache.insert(invoice_conversion_job.xslt_key.clone(), compiled);
            xslt_cache.get(&invoice_conversion_job.xslt_key).unwrap()
        };

        match engine.transform(compiled, &invoice_conversion_job.xml_data) {
            Ok(html_bytes) => {
                let filename = filename_in_zip(
                    &invoice_conversion_job.item,
                    &filename_in_zip_mode,
                    docs_count,
                );

                if let Err(e) =
                    zip.write_to_zip(&filename, html_bytes)
                        .map_err(|e| InvConvError::ZipIOError {
                            sira_no: invoice_conversion_job.item.sira_no.unwrap_or(0).to_string(),
                            source: e,
                        })
                {
                    log_error(&e);
                    if e.is_fatal() {
                        return Err(e);
                    }

                    close_zip_and_return(
                        &mut zip,
                        docs_count,
                        total_html_bytes,
                        last_processed_sira_no,
                        false,
                    )?;
                }
            }
            Err(e) => {
                log_error(&e);
                if e.is_fatal() {
                    return Err(e);
                }

                close_zip_and_return(
                    &mut zip,
                    docs_count,
                    total_html_bytes,
                    last_processed_sira_no,
                    false,
                )?;
            }
        }

        /*
        match transform_xml_to_html_bytes(&xml_bytes) {
            Ok(html) => {
                let filename = filename_in_zip(&item, FilenameInZipMode::default(), docs_count);
                if let Err(e) = zip.start_file(filename, zip_opts) {
                    let err = DocProcessingError::ZipError {
                        sira_no: item.sira_no.unwrap_or(0).to_string(),
                        source: e,
                    };
                    if err.is_fatal() {
                        return Err(err);
                    } else {
                        continue;
                    }
                }
                zip.write_all(&html).unwrap();
                docs_count += 1;
            }
            Err(err) => {
                if err.is_fatal() {
                    return Err(err);
                } else {
                    continue;
                }
            }
        }*/
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

fn close_zip_and_return(
    zip: &mut ZipWriter<std::fs::File>,
    docs_count: u8,
    size: u64,
    last_processed_sira_no: u64,
    request_fully_completed: bool,
) -> Result<InvoiceConversionResult, InvConvError> {
    let mut tmp = zip.finish().unwrap();
    let file_size = tmp.metadata().unwrap().len();
    let mut buf = Vec::with_capacity(file_size as usize);
    tmp.seek(SeekFrom::Start(0)).unwrap();
    tmp.read_to_end(&mut buf).unwrap();

    Ok(InvoiceConversionResult {
        data: buf,
        docs_count: docs_count,
        size: size,
        last_processed_sira_no: Some(last_processed_sira_no),
        request_fully_completed: request_fully_completed,
    })
}

fn log_error(e: &InvConvError) {
    //log::error!("Error processing sira_no {}: {:?}", sira_no, e);
}
