use crate::utils::download_request::download_request_types::{
    DownloadDocRequest, DownloadFormat, DownloadType,
};
use crate::utils::errors::download_request_errors::{
    DownloadRequestError, ErrCtx as DownloadRequestErrCtx,
};
use crate::utils::{
    database_manager::init_database, incoming_invoice::incoming_invoice_rec::IncomingInvoiceRec,
};
use tiberius::Query;

impl DownloadDocRequest {
    fn extract_dbname(&self) -> Result<String, DownloadRequestError> {
        //Ok(format!("uut_{}_{}", 24, 6))
        return Err(DownloadRequestError::CanNotExtractDatabaseName(
            "path in incomin invoices".to_string(),
        ));
    }

    pub fn validate_download_type_and_format(&self) -> Result<(), DownloadRequestError> {
        use DownloadFormat::*;
        use DownloadType::*;

        match (self.download_type, self.format) {
            // ✅ valid combinations
            (Html, Zip)
            | (Html, Gzip)
            | (Pdf, Zip)
            | (Pdf, Gzip)
            | (Ubl, Zip)
            | (Ubl, Gzip)
            | (UblXsltSeparate, Zip)
            | (UblXsltSeparate, Gzip) => Ok(()),

            // 🚫 anything else
            _ => Err(DownloadRequestError::InvalidTypeFormat {
                downloadtype: self.download_type,
                format: self.format,
            }),
        }
    }

    pub async fn get_incoming_invoice_recs(
        &self,
        pool: &init_database::ConnectionPool,
    ) -> Result<Vec<IncomingInvoiceRec>, DownloadRequestError> {
        self.validate_download_type_and_format()?;
        let db_name = self.extract_dbname().ctx("get_incoming_invoice_recs")?;
        let sql_sentence = format!(
            "SELECT TOP 100 UUID, INVOICE_ID, RECEIVER_CONTACT, SIRA_NO, PATH
             FROM {}.dbo.INCOMING_INVOICE
             WHERE RECEIVER_CONTACT = @P1 AND SIRA_NO > @P2
             ORDER BY SIRA_NO ASC",
            db_name // e.g. "uut_24_6" or "uut_25_1"
        );
        let mut query = Query::new(sql_sentence);
        query.bind(self.source_vkntckn.clone());
        query.bind(self.after_this);

        let mut conn = pool
            .get()
            .await
            .map_err(DownloadRequestError::from)
            .ctx("get_incoming_invoice_recs")?;

        let stream = query
            .query(&mut *conn)
            .await
            .map_err(DownloadRequestError::from)
            .ctx("get_incoming_invoice_recs:Query")?;
        let rows = stream
            .into_first_result()
            .await
            .map_err(DownloadRequestError::from)
            .ctx("get_incoming_invoice_recs:stream")?;

        let mut incoming_invoice_recs: Vec<IncomingInvoiceRec> = Vec::with_capacity(rows.len());
        for row in rows {
            // Try to extract all fields
            let Some(uuid) = row.get::<&str, _>(0) else {
                eprintln!("⚠️ Skipping row: missing UUID");
                continue;
            };
            let Some(invoice_id) = row.get::<&str, _>(1) else {
                eprintln!("⚠️ Skipping row: missing INVOICE_ID");
                continue;
            };
            let Some(receiver_contact) = row.get::<&str, _>(2) else {
                eprintln!("⚠️ Skipping row: missing RECEIVER_CONTACT");
                continue;
            };
            let Some(sira_no_i64) = row.get::<i64, _>(3) else {
                eprintln!("⚠️ Skipping row: missing SIRA_NO");
                continue;
            };
            let Some(path) = row.get::<&str, _>(4) else {
                eprintln!("⚠️ Skipping row: missing PATH");
                continue;
            };

            incoming_invoice_recs.push(IncomingInvoiceRec {
                uuid: uuid.to_string(),
                invoice_id: invoice_id.to_string(),
                receiver_contact: receiver_contact.to_string(),
                sira_no: sira_no_i64 as u64,
                path: path.to_string(),
                //s3: None,
            });
        }

        println!(
            "✅ Successfully parsed {} records",
            incoming_invoice_recs.len()
        );
        Ok(incoming_invoice_recs)
    }
}
