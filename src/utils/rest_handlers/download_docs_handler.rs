use axum::{Json, extract::State, http::StatusCode};

use base64::{Engine as _, engine::general_purpose};
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;
use std::sync::Arc;

use crate::utils::download_request::download_request_types::DownloadDocRequest;
use crate::utils::errors::download_request_errors::DownloadRequestError;
use crate::utils::incoming_invoice::incoming_invoice_rec;
use crate::utils::rest_handlers::rest_responses::{DownloadResponse, ErrorResponse};
use crate::utils::{
    appstate::appstate,
    get_and_process_invoices::process_invoices_types::GetAndProcessInvoicesRequest,
};

pub async fn download_docs_handler(
    State(state): State<appstate::SharedState>,
    Json(request): Json<DownloadDocRequest>,
) -> Result<Json<DownloadResponse>, (StatusCode, Json<ErrorResponse>)> {
    println!(
        "📥 Download request: vkntckn={}, after_this={}, type={:?}",
        request.source_vkntckn, request.after_this, request.download_type
    );
    let request: Arc<DownloadDocRequest> = Arc::new(request);

    // Fetch invoice records from database
    /*
    let invoices = request
        .get_incoming_invoice_recs(&state.db_pools.object_pool)
        .await
        .map_err(|e| {
            eprintln!("❌ Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "DatabaseError".to_string(),
                    message: format!("Failed to fetch invoices: {}", e),
                }),
            )
        })?;
        */

    let invoices = match request
        .get_incoming_invoice_recs(&state.db_pools.object_pool)
        .await
    {
        Ok(invoices) => invoices,

        Err(e) => {
            match e {
                DownloadRequestError::InvalidTypeFormat {
                    downloadtype,
                    format,
                } => {
                    return Err((
                        StatusCode::NOT_FOUND,
                        Json(ErrorResponse {
                            error: "Wrong type and format".to_string(),
                            //message: "Allowed types Html, Pdf, Ubl, UblXsltSeparate and formats Zip, Gzip but i got {downloadtype:?} and {format:?}".to_string(),
                            message: format!(
                                "Allowed types Html, Pdf, Ubl, UblXsltSeparate and formats Zip, Gzip but i got {downloadtype:?} and {format:?}"
                            ),
                        }),
                    ));
                }
                _ => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "Internal Error".to_string(),
                            message: format!("Error source: {}", e),
                        }),
                    ));
                }
            }
        }
    };
    println!("✅ Found {} invoice records", invoices.len());

    let process_invoices_req = GetAndProcessInvoicesRequest {
        request: request.clone(),
        invoices: invoices,
    };
    match process_invoices_req.process().await {
        Ok(result) => {
            println!(
                "✅ Processed {} invoices, size: {} bytes",
                result.record_count, result.size_bytes
            );
            return Ok(Json(DownloadResponse {
                data: result.data,
                filename: result.filename,
                record_count: result.record_count,
                size_bytes: result.size_bytes,
            }));
        }
        Err(e) => {
            eprintln!("❌ Processing error: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "ProcessingError".to_string(),
                    message: format!("Failed to process invoices: {}", e),
                }),
            ));
        }
    }

    // This func is just debugging the json conversion and returning http response
    /*
    match convert_json_and_return_http_response(&request, &invoices) {
        Ok(response) => return Ok(response),
        Err(e) => {
            eprintln!("Error in converting json : {}", e.0,);
            return Err(e);
        }
    }
    */
}

fn compress_data(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}
fn convert_json_and_return_http_response(
    request: &DownloadDocRequest,
    invoices: &Vec<incoming_invoice_rec::IncomingInvoiceRec>,
) -> Result<Json<DownloadResponse>, (StatusCode, Json<ErrorResponse>)> {
    let json_data = serde_json::to_string_pretty(&invoices).map_err(|e| {
        eprintln!("❌ JSON serialization error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "SerializationError".to_string(),
                message: format!("Failed to serialize data: {}", e),
            }),
        )
    })?;

    // Compress with gzip
    let compressed_data = compress_data(json_data.as_bytes()).map_err(|e| {
        eprintln!("❌ Compression error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "CompressionError".to_string(),
                message: format!("Failed to compress data: {}", e),
            }),
        )
    })?;

    // Encode to base64
    let base64_data = general_purpose::STANDARD.encode(&compressed_data);

    // Generate filename
    let filename = format!(
        "invoices_{}_{}.json.gz",
        request.source_vkntckn,
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );

    println!(
        "✅ Compressed {} records, size: {} bytes",
        invoices.len(),
        compressed_data.len()
    );
    Ok(Json(DownloadResponse {
        data: base64_data,
        filename,
        record_count: invoices.len(),
        size_bytes: compressed_data.len(),
    }))
}
