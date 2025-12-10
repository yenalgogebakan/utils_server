# UTILS SERVER

This is a utility server for various backend services. It provides APIs for common functionalities that can be used across different applications.Currently XML to HTML and PDF conversion services are implemented.

**utils_server** has two entry points REST API and gRPC API.

- main.rs - does nothing
- bin/rest_server.rs - REST API server
- bin/grpc_server.rs - gRPC API server

Both servers will init the database and object store pools and accept requests on tokio threads. 

## invoice_conversion_manager
**utils/convert_invoices/invoice_conversion_manager.rs**  - This module handles the conversion of XML invoices to HTML and PDF formats. It accepts 

```rust
/// ----- Info about an invoice to be converted -----
#[derive(Debug, Clone)]
pub struct InvoiceItemForConversion {
    pub object_id: String,
    pub sira_no: Option<u64>,
    pub invoice_no: Option<String>,
}

/// ----- Batch of invoices to be converted -----
#[derive(Debug, Clone)]
pub struct InvoicesForConversion {
    pub target_type: TargetType,
    pub target_compression_type: TargetCompressionType,
    pub year: String,
    pub filename_in_zip: FilenameInZipMode,

    /// Items to fetch/process
    pub items: Vec<InvoiceItemForConversion>,
}
```
either rest or gRpc services will convert the incoming requests to this struct and call the invoice conversion manager to process the request.

invoice_conversion_manager returns

```rust
/// ----- Success Result, data is the zipped file -----
#[derive(Debug, Clone, Default)]
pub struct InvoiceConversionResult {
    pub data: Vec<u8>,
    pub docs_count: u32,
    pub size: u64,
    pub last_processed_sira_no: Option<u64>,
    pub request_fully_completed: bool,
}

/// ----- Error Response -----
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InvoiceConversionError {
    pub error_code: i32,
    pub error_msg: String,
}
impl From<InvConvError> for InvoiceConversionError {
    fn from(error: InvConvError) -> Self {
        Self {
            error_code: error.error_code(),
            error_msg: error.to_string(), // Uses Display from thiserror
        }
    }
}
```
where 
- data : Zipped file containing converted invoices
- docs_count : Number of documents converted
- size : Size of the zipped file
- last_processed_sira_no : Last processed invoice number
- request_fully_completed : Indicates if the entire request was processed successfully


## rest_handlers
**get_invoices_handler.rs** is the main rest handler for invoice conversion. In the future other utility services can be added here. Current handler will impose a limit of max  const MAX_BLOCKING_TASKS: usize = 64 in rest_server.rs. rest handler also implements a cancellation token to cancel the tokio task when the client drops the connection
