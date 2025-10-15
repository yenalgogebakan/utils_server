use std::sync::Arc;

use crate::utils::download_request::download_request_types::{
    DownloadDocRequest, DownloadFormat, DownloadType,
};
use crate::utils::incoming_invoice::incoming_invoice_rec::IncomingInvoiceRec;
use crate::utils::object_store::object_store::Store;

use super::process_invoices_types::{GetAndProcessInvoicesRequest, GetAndProcessInvoicesResult};

impl GetAndProcessInvoicesRequest {
    pub fn new(request: Arc<DownloadDocRequest>, invoices: Vec<IncomingInvoiceRec>) -> Self {
        Self {
            request: request,
            invoices: invoices,
        }
    }
    pub fn len(&self) -> usize {
        self.invoices.len()
    }
    pub fn is_empty(&self) -> bool {
        self.invoices.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = &IncomingInvoiceRec> {
        self.invoices.iter()
    }

    pub async fn process(
        &self,
        object_store: &Store,
    ) -> anyhow::Result<GetAndProcessInvoicesResult> {
        // Placeholder for processing logic
        println!(
            "Processing {} invoices for request: {}",
            self.len(),
            self.request
        );
        for invoice in &self.invoices {
            println!("Processing invoice: {}", invoice);
            // Add actual processing logic here
        }

        match (self.request.download_type, self.request.format) {
            (DownloadType::Html, DownloadFormat::Zip) => {
                match self.process_into_html(object_store).await {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        eprintln!("❌ Error processing invoices: {}", e);
                        Err(e)
                    }
                }
            }
            _ => Err(anyhow::anyhow!(
                "Processing for the specified download type and format is not implemented yet."
            )),
        }
    }
}
