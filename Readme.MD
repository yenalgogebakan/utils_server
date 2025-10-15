REST_SERVER binds the route to the handler function : download_docs_handler
this handler creates a download_request by using axum extractor. DownloadDocRequest
This struct has a method get_incoming_invoice_recs which collect the related infos into pub struct
IncomingInvoiceRec {
    pub uuid: String,
    pub invoice_id: String,
    pub receiver_contact: String,
    pub sira_no: u64,
    pub path: String,
    //pub s3: Option<Box<S3Object>>,
}

At this point we have Vec<IncomingInvoiceRec> which will be processed one by one. Now they are passed to get_and_process_invoices jobs
To do this Vec<IncomingInvoiceRec> and download_request (DownloadDocRequest) are used to contruct a
pub struct GetAndProcessInvoicesRequest {
    pub request: Arc<DownloadDocRequest>,
    pub invoices: Vec<IncomingInvoiceRec>,
}

which has an implementation process. In process the actual job is dispatched process_into_html, process_into_pdf etc.
