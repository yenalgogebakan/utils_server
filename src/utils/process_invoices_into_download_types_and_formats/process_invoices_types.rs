#[derive(Debug)]
pub struct ProcessInvoicesAccordingtoTypesAndFormatsResult {
    pub data: String, // base64 encoded zip
    pub filename: String,
    pub record_count: usize,
    pub size_bytes: usize,
}
