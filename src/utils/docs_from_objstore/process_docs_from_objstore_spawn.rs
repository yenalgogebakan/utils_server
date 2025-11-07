use tokio::sync::OwnedSemaphorePermit;
use tokio_util::sync::CancellationToken;

use super::get_ubl_decomp_from_objstore::get_ubl_decomp_from_object_store;
use crate::utils::appstate::appstate::SharedState;
use crate::utils::common::download_types_and_formats::{
    DownloadFormat, DownloadType, FilenameInZipMode,
};
use crate::utils::errors::doc_processing_errors::{DocProcessingError, ErrCtx};
use crate::utils::object_store::{object_store::Store, opendal_mssql_wrapper::ObjectStoreRecord};
use crate::utils::rest_handlers::docs_from_objstore_spawn_handler::{
    DocRequestItem, DocsFromObjStoreReq, DocsFromObjStoreResponse,
};
use std::io::{Read, Seek, SeekFrom, Write};
use tempfile::tempfile;
use zip::{ZipWriter, write::FileOptions};

pub fn process_docs_from_objstore_spawn(
    req: DocsFromObjStoreReq,
    state: SharedState,
    _permit: OwnedSemaphorePermit, // keep as parameter so it's dropped when this fn returns
    cancel: CancellationToken,
) -> Result<DocsFromObjStoreResponse, DocProcessingError> {
    // We are in the sync thread, in which we will read ubls and convert them to HTML one by one

    let tmp = tempfile().unwrap();
    let mut zip: ZipWriter<std::fs::File> = ZipWriter::new(tmp);
    let zip_opts = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut last_processed_sira_no = 0u64;
    let mut total_html_bytes = 0u64;
    let mut docs_count = 0u32;
    let filename_in_zip_mode: FilenameInZipMode = req.filename_in_zip;

    for (idx, item) in req.items.iter().enumerate() {
        // check for cancellation, connection closed, etc.
        if cancel.is_cancelled() {
            return Err(DocProcessingError::ClientDisconnectedError(
                "Client disconnected, cancelling processing".to_string(),
            ))
            .ctx("process_docs_from_objstore_spawn:cancelled");
        }

        // Your real transform here (bytes!)
        match transform_xml_to_html(&state.object_store, &req.year, item) {
            Ok(html_bytes) => {
                let filename = filename_in_zip(&item, filename_in_zip_mode, docs_count);
                if let Err(e) = zip
                    .start_file(filename, zip_opts)
                    .map_err(|e| DocProcessingError::ZipError {
                        sira_no: item.sira_no.unwrap_or(0).to_string(),
                        source: e,
                    })
                    .ctx("process_docs_from_objstore_spawn:zip.start_file")
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
                if let Err(e) = zip
                    .write_all(&html_bytes)
                    .map_err(|e| DocProcessingError::ZipIOError {
                        sira_no: item.sira_no.unwrap_or(0).to_string(),
                        source: e,
                    })
                    .ctx("process_docs_from_objstore_spawn:zip.write_all")
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

                last_processed_sira_no = item.sira_no.unwrap_or(0);
                total_html_bytes += html_bytes.len() as u64;
                docs_count += 1;
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
    }

    //Close ZIP
    close_zip_and_return(
        &mut zip,
        docs_count,
        total_html_bytes,
        last_processed_sira_no,
        true,
    )
}

fn filename_in_zip(
    item: &DocRequestItem,
    filename_in_zip_mode: FilenameInZipMode,
    docs_count: u32,
) -> String {
    match filename_in_zip_mode {
        FilenameInZipMode::ExtractFromObjID => "Invoice_001".to_string(),
        FilenameInZipMode::IncludedInRequest => match &item.invoice_no {
            Some(value) => format!("Fat_{}", value),
            None => format!("Fat_{}", docs_count),
        },
        FilenameInZipMode::UseSiraNo => match item.sira_no {
            Some(value) => format!("Fat_{}", value),
            None => format!("Fat_00000000"),
        },
        FilenameInZipMode::StartFromInvoiceOne => format!("Fat_{}", docs_count),
    }
}

fn close_zip_and_return(
    zip: &mut ZipWriter<std::fs::File>,
    docs_count: u32,
    size: u64,
    last_processed_sira_no: u64,
    request_fully_completed: bool,
) -> Result<DocsFromObjStoreResponse, DocProcessingError> {
    let mut tmp = zip.finish().unwrap();
    let file_size = tmp.metadata().unwrap().len();
    let mut buf = Vec::with_capacity(file_size as usize);
    tmp.seek(SeekFrom::Start(0)).unwrap();
    tmp.read_to_end(&mut buf).unwrap();

    Ok(DocsFromObjStoreResponse {
        data: buf,
        docs_count: docs_count,
        size: size,
        last_processed_sira_no: Some(last_processed_sira_no),
        request_fully_completed: request_fully_completed,
    })
}

fn transform_xml_to_html(
    object_store: &Store,
    year: &String,
    _item: &DocRequestItem,
) -> Result<Vec<u8>, DocProcessingError> {
    get_ubl_decomp_from_object_store(object_store, year, &_item.object_id).await;

    Ok(b"<html><body>ok</body></html>".to_vec())
}

fn log_error(e: &DocProcessingError) {
    //log::error!("Error processing sira_no {}: {:?}", sira_no, e);
}
