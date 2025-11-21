use crate::utils::appstate::appstate::SharedState;
use crate::utils::common::target_types_and_formats::{
    FilenameInZipMode, TargetCompressionType, TargetType,
};
use crate::utils::convert_invoices::invoice_conversion_manager::{
    InvoiceConversionJob, InvoiceConversionResult,
};
use crate::utils::errors::invoice_conversion_errors::{ErrCtx, InvConvError};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom, Write};
use tempfile::tempfile;
use tokio::sync::mpsc;
use tokio_util::bytes;
use tokio_util::sync::CancellationToken;
use zip::{CompressionMethod, ZipWriter, write::FileOptions};

/// ---- blocking worker ----
pub fn convert_and_zip(
    mut rx: mpsc::Receiver<InvoiceConversionJob>,
    state: SharedState,
    worker_cancellation_token: CancellationToken,
    target_type: TargetType,
    target_compression_type: TargetCompressionType,
    filename_in_zip_mode: FilenameInZipMode,
) -> Result<InvoiceConversionResult, InvConvError> {
    let tmp = tempfile().unwrap();
    let mut zip = ZipWriter::new(tmp);
    let zip_opts = FileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut docs_count = 0u32;
    let mut last_processed_sira_no = 0u64;
    let mut total_html_bytes = 0u64;

    while let Some(invoice_conversion_job) = rx.blocking_recv() {
        if worker_cancellation_token.is_cancelled() {
            return Err(InvConvError::ClientDisconnectedError(
                "Client disconnected, task canceled".to_string(),
            ))
            .ctx("convert_and_zip:process cancelled");
        }

        let mut xslt_cache: HashMap<String, bytes::Bytes> = HashMap::with_capacity(4);
        match transform_xml_to_html_bytes(&xml_bytes) {
            Ok(html) => {
                let filename = filename_in_zip(&item, FilenameInZipMode::Default, docs_count);
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
        }
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
