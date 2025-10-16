use std::sync::Arc;

use crate::utils::download_request::download_request_types::{
    DownloadDocRequest, DownloadFormat, DownloadType,
};
use crate::utils::incoming_invoice::incoming_invoice_rec::IncomingInvoiceRec;
use crate::utils::object_store::object_store::Store;

use super::process_invoices_types::{GetAndProcessInvoicesRequest, GetAndProcessInvoicesResult};
use crate::utils::errors::processing_errors::ProcessingError;
use crate::utils::get_and_process_invoices::process_single_invoice_into_html::process_single_invoice_into_html;

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

    // Decides the convertion type and calls the relevant function
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
                match self.process_invoices_into_html(object_store).await {
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

    pub async fn process_invoices_into_html(
        &self,
        object_store: &Store,
    ) -> anyhow::Result<GetAndProcessInvoicesResult> {
        /* We are working on this
        // pub struct GetAndProcessInvoicesRequest {
            pub request: Arc<DownloadDocRequest>,
            pub invoices: Vec<IncomingInvoiceRec>,
        }
        For each invoice we will get the XML from the object store, convert it to HTML, and then package them as needed.
        */

        let mut processing_errors: Vec<ProcessingError> = Vec::new();
        for (index, invoice) in self.invoices.iter().enumerate() {
            println!("Processing invoice number: {}", index);

            if let Some(year) = invoice.extract_year_as_string() {
                println!("Extracted year: {}", year);

                let invoice_html: Vec<u8> = process_single_invoice_into_html(
                    object_store,
                    &invoice.path,
                    &year,
                    &processing_errors,
                )
                .await?;
            } else {
                let error_message = format!("Could not extract year from path: '{}'", invoice.path);
                let processing_error = ProcessingError {
                    invoice_id: invoice.invoice_id.clone(),
                    error_code: Some("NOYEARFROMPATH".to_string()),
                    message: error_message,
                };
                //log_error(&processing_error);
                processing_errors.push(processing_error);
                continue; // Skip this invoice if year extraction fails
            }
        }

        Ok(GetAndProcessInvoicesResult {
            data: "deneme".to_string(),
            filename: "deneme".to_string(),
            record_count: 5,
            size_bytes: 10,
        })
    }
}
