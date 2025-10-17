use super::process_invoices_types::ProcessInvoicesAccordingtoTypesAndFormatsResult;
use crate::utils::common::download_types_and_formats::{DownloadFormat, DownloadType};
use crate::utils::incoming_invoice::incoming_invoice_rec::IncomingInvoiceRec;
use crate::utils::object_store::object_store::Store;
use crate::utils::process_invoices_into_download_types_and_formats::process_invoices_into_html::process_invoices_into_html;

// Decides the convertion type and calls the relevant function
pub async fn process_invoices_accordingto_types_and_formats(
    object_store: &Store,
    invoices: &Vec<IncomingInvoiceRec>,
    download_type: DownloadType,
    format: DownloadFormat,
) -> anyhow::Result<ProcessInvoicesAccordingtoTypesAndFormatsResult> {
    for invoice in invoices {
        println!("Processing invoice: {}", invoice);
        // Add actual processing logic here
    }

    match download_type {
        DownloadType::Html => {
            match process_invoices_into_html(object_store, invoices, format).await {
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
