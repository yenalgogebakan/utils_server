//use crate::utils::object_store_old::s3object::S3Object;
use std::fmt;

#[derive(Default, Debug, serde::Serialize)]
pub struct IncomingInvoiceRec {
    pub uuid: String,
    pub invoice_id: String,
    pub receiver_contact: String,
    pub sira_no: u64,
    pub path: String,
    //pub s3: Option<Box<S3Object>>,
}

impl fmt::Display for IncomingInvoiceRec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let uuid_str = if self.uuid.is_empty() {
            "unset".to_string()
        } else {
            self.uuid.clone()
        };
        let invoice_id_str = if self.invoice_id.is_empty() {
            "unset".to_string()
        } else {
            self.invoice_id.clone()
        };
        let receiver_contact_str = if self.receiver_contact.is_empty() {
            "unset".to_string()
        } else {
            self.receiver_contact.clone()
        };
        let sira_no_str = if self.sira_no == 0 {
            "0".to_string()
        } else {
            self.sira_no.to_string()
        };

        let path_str = if self.path.is_empty() {
            "unset".to_string()
        } else {
            self.path.clone()
        };

        write!(
            f,
            "IncomingInvoiceRec {{ uuid: {}, invoice_id: {}, receiver_contact: {}, sira_no: {}, path: {} }}",
            uuid_str, invoice_id_str, receiver_contact_str, sira_no_str, path_str
        )
    }
}

impl IncomingInvoiceRec {
    pub fn new(
        uuid: impl Into<String>,
        invoice_id: impl Into<String>,
        receiver_contact: impl Into<String>,
        sira_no: u64,
        path: impl Into<String>,
    ) -> Self {
        Self {
            uuid: uuid.into(),
            invoice_id: invoice_id.into(),
            receiver_contact: receiver_contact.into(),
            sira_no,
            path: path.into(),
            //s3: None,
        }
    }
}
