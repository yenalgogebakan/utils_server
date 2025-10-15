use std::sync::Arc;

use crate::utils::download_request::download_request_types::DownloadDocRequest;
use crate::utils::incoming_invoice::incoming_invoice_rec::IncomingInvoiceRec;

#[derive(Debug)]
pub struct GetAndProcessInvoicesRequest {
    pub request: Arc<DownloadDocRequest>,
    pub invoices: Vec<IncomingInvoiceRec>,
}

#[derive(Debug)]
pub struct GetAndProcessInvoicesResult {
    pub data: String, // base64 encoded zip
    pub filename: String,
    pub record_count: usize,
    pub size_bytes: usize,
}
