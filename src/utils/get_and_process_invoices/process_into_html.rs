use super::process_invoices_types::{GetAndProcessInvoicesRequest, GetAndProcessInvoicesResult};
use crate::utils::errors::processing_errors::ProcessingError;
use crate::utils::object_store::{object_store::Store, opendal_mssql_wrapper::ObjectStoreRecord};

impl GetAndProcessInvoicesRequest {
    pub async fn process_into_html(
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

                let object_store_rec: ObjectStoreRecord =
                    object_store.get("ubls", &invoice.path, &year).await?;
                // Now we have object content (which is Ubl now) and orginal size and compressed size
                //
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
