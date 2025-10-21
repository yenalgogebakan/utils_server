//use crate::utils::common::processed_invoice_types::ProcessedInvoice;
use crate::utils::common::{
    comp_decompress::xz_decompress, download_types_and_formats::DownloadFormat,
    processed_invoice_types::ProcessedInvoice, san_desanitize::sanitize_fast,
};
use crate::utils::errors::process_errors::ProcessError;
use crate::utils::errors::process_errors::ProcessingError;
use crate::utils::incoming_invoice::incoming_invoice_rec::IncomingInvoiceRec;
use crate::utils::object_store::{object_store::Store, opendal_mssql_wrapper::ObjectStoreRecord};
use crate::utils::process_invoices_into_download_types_and_formats::process_invoices_types::ProcessInvoicesAccordingtoTypesAndFormatsResult;

pub async fn process_invoices_into_html(
    object_store: &Store,
    invoices: &Vec<IncomingInvoiceRec>,
    format: DownloadFormat,
) -> anyhow::Result<ProcessInvoicesAccordingtoTypesAndFormatsResult> {
    let mut processing_errors: Vec<ProcessingError> = Vec::new();
    let mut processed_invoices: Vec<ProcessedInvoice> = Vec::new();
    let bucket = "ubls".to_string();
    for (index, invoice) in invoices.iter().enumerate() {
        println!("Processing invoice number: {}", index);

        if let Some(year) = invoice.extract_year_as_string() {
            println!("Extracted year: {}", year);
            let path = invoice.path.replace("/", "");
            let object_id = path + ".xz";
            let processed_invoice: ProcessedInvoice = match process_single_invoice_into_html(
                object_store,
                &invoice.invoice_id,
                &bucket,
                &object_id,
                &year,
            )
            .await
            {
                Ok(processed_invoice) => processed_invoice,
                Err(e) => match e {
                    ProcessError::UblNotFoundInObjectStore(object_id) => {
                        let error_message =
                            format!("UBL not found in object store for  path='{}'", object_id);
                        let processing_error = ProcessingError {
                            invoice_id: invoice.invoice_id.clone(),
                            error_code: Some("UBLNOTFOUND".to_string()),
                            message: error_message.clone(),
                        };
                        //log_error(&processing_error);
                        processing_errors.push(processing_error);
                        continue; // Skip this invoice on UBL not found
                    }
                    ProcessError::NonUtfCharError(_) => {
                        let error_message = format!(
                            "Non-UTF characters found in invoice_id='{}'",
                            invoice.invoice_id
                        );
                        let processing_error = ProcessingError {
                            invoice_id: invoice.invoice_id.clone(),
                            error_code: Some("NONUTFCHAR".to_string()),
                            message: error_message.clone(),
                        };
                        //log_error(&processing_error);
                        processing_errors.push(processing_error);
                        continue; // Skip this invoice on non-UTF char error
                    }
                    _ => {
                        let error_message = format!(
                            "Error processing invoice_id='{}': {}",
                            invoice.invoice_id, e
                        );
                        let processing_error = ProcessingError {
                            invoice_id: invoice.invoice_id.clone(),
                            error_code: Some("PROCESSINGERROR".to_string()),
                            message: error_message.clone(),
                        };
                        //log_error(&processing_error);
                        processing_errors.push(processing_error);
                        continue; // Skip this invoice on other errors
                    }
                },
            };
            processed_invoices.push(processed_invoice);
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

    Ok(ProcessInvoicesAccordingtoTypesAndFormatsResult {
        data: "deneme".to_string(),
        filename: "deneme".to_string(),
        record_count: 5,
        size_bytes: 10,
    })
}
const DECOMPRESS_ASYNC_THRESHOLD: i32 = 2 * 1024 * 1024; // 2MB
pub async fn process_single_invoice_into_html(
    object_store: &Store,
    invoice_id: &str,
    bucket: &String,
    path_in_object_store: &String, //key
    year: &String,
) -> Result<ProcessedInvoice, ProcessError> {
    if !object_store
        .object_exists(bucket, path_in_object_store, year)
        .await?
    {
        return Err(ProcessError::UblNotFoundInObjectStore(
            path_in_object_store.to_string(),
        ));
    }
    let object_store_rec: ObjectStoreRecord = object_store
        .get("ubls", &path_in_object_store, &year)
        .await?;
    let decompressed = if object_store_rec.original_size >= DECOMPRESS_ASYNC_THRESHOLD {
        // Large file: offload to blocking thread
        tokio::task::spawn_blocking(move || {
            xz_decompress(
                &object_store_rec.objcontent,
                object_store_rec.original_size as usize,
            )
        })
        .await
        .map_err(ProcessError::TaskJoin)??
    } else {
        // Small file: decompress inline
        xz_decompress(
            &object_store_rec.objcontent,
            object_store_rec.original_size as usize,
        )?
    };

    let sanitized = sanitize_fast(&decompressed)?;

    Ok(ProcessedInvoice { data: Vec::new() })
}
/*
* let xml = b"<name>John&#x1F;Doe</name>";
let result = sanitize_fast(xml);

match &result {
    Cow::Borrowed(_) => println!("Zero-copy"),
    Cow::Owned(v) => {
        println!("Sanitization happened");
        // v contains: b"<name>John-sanitized-x1F--Doe</name>"
    }
}
*/
