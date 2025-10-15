use serde::Serialize;

// Response wrapper for base64 encoded zip
#[derive(Serialize, Debug)]
pub struct DownloadResponse {
    pub data: String, // base64 encoded zip
    pub filename: String,
    pub record_count: usize,
    pub size_bytes: usize,
}

// Error response
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}
