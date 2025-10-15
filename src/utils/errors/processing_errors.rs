#[derive(Debug, Clone, serde::Serialize)]
pub struct ProcessingError {
    pub invoice_id: String,
    pub error_code: Option<String>,
    pub message: String,
}
