use axum::{Json, extract::State, http::StatusCode};

//use base64::{Engine as _, engine::general_purpose};
//use flate2::Compression;
//use flate2::write::GzEncoder;
//use std::io::Write;
use serde::Serialize;

use crate::utils::appstate::appstate;
use crate::utils::common::download_types_and_formats::{DownloadFormat, DownloadType};
use crate::utils::incoming_invoice::get_incoming_invoice_recs_afterthis::get_incoming_invoice_recs_afterthis;
use crate::utils::process_invoices_into_download_types_and_formats::process_invoices_accordingto_types_and_formats::process_invoices_accordingto_types_and_formats;

// REST request and response structs
#[derive(Serialize, serde::Deserialize, Debug, Clone)]
pub struct DownloadDocsRequest {
    pub source_vkntckn: String,
    pub after_this: i64,
    pub download_type: DownloadType,
    pub format: DownloadFormat,
}
impl std::fmt::Display for DownloadDocsRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Source vkntckn: {} after count: {} download type: {} format: {}",
            self.source_vkntckn, self.after_this, self.download_type, self.format
        )
    }
}

// Response wrapper for base64 encoded zip
#[derive(Serialize, Debug)]
pub struct DownloadDocsResponse {
    pub data: String, // base64 encoded zip
    pub filename: String,
    pub record_count: usize,
    pub size_bytes: usize,
}
// Error response
#[derive(Serialize)]
pub struct DownloadDocsErrorResponse {
    pub error: String,
    pub message: String,
}

pub async fn download_docs_handler(
    State(state): State<appstate::SharedState>,
    Json(request): Json<DownloadDocsRequest>,
) -> Result<Json<DownloadDocsResponse>, (StatusCode, Json<DownloadDocsErrorResponse>)> {
    println!(
        "📥 Download request: vkntckn={}, after_this={}, type={:?}",
        request.source_vkntckn, request.after_this, request.download_type
    );

    // At this point, we have the request and can query the database for invoice records. The dbname for INCOMING_INVOICES is "uut_24_6"
    let dbname = "uut_24_6";
    let invoices = match get_incoming_invoice_recs_afterthis(
        &state.db_pools.incoming_invoice_pool,
        &dbname,
        &request.source_vkntckn,
        request.after_this,
    )
    .await
    {
        Ok(invoices) => invoices,
        Err(e) => match e {
            _ => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(DownloadDocsErrorResponse {
                        error: "Internal Error".to_string(),
                        message: format!("Error source: {}", e),
                    }),
                ));
            }
        },
    };
    println!("✅ Found {} invoice records", invoices.len());
    if invoices.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(DownloadDocsErrorResponse {
                error: "NoInvoices".to_string(),
                message: format!(
                    "No invoices found for vkntckn {} after sirano {}",
                    request.source_vkntckn, request.after_this
                ),
            }),
        ));
    }

    match process_invoices_accordingto_types_and_formats(
        &state.object_store,
        &invoices,
        request.download_type,
        request.format,
    )
    .await
    {
        Ok(result) => {
            println!(
                "✅ Processed {} invoices, size: {} bytes",
                result.record_count, result.size_bytes
            );
            return Ok(Json(DownloadDocsResponse {
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
                Json(DownloadDocsErrorResponse {
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
/*
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
*/
