use super::process_invoices_types::{GetAndProcessInvoicesRequest, GetAndProcessInvoicesResult};

impl GetAndProcessInvoicesRequest {
    pub async fn process_into_html(&self) -> anyhow::Result<GetAndProcessInvoicesResult> {
        Ok(GetAndProcessInvoicesResult {
            data: "deneme".to_string(),
            filename: "deneme".to_string(),
            record_count: 5,
            size_bytes: 10,
        })
    }
}
