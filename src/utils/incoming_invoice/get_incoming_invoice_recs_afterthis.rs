use super::incoming_invoice_rec::IncomingInvoiceRec;
use crate::utils::database_manager::init_database;
use crate::utils::errors::db_errors::{DbError, ErrCtx as DbErrCtx};
use tiberius::Query;

pub async fn get_incoming_invoice_recs_afterthis(
    pool: &init_database::ConnectionPool,
    dbname: &str,
    source_vkntckn: &str,
    after_this: i64,
) -> Result<Vec<IncomingInvoiceRec>, DbError> {
    let sql_sentence = format!(
        "SELECT TOP 100 UUID, INVOICE_ID, RECEIVER_CONTACT, SIRA_NO, PATH
         FROM {}.dbo.INCOMING_INVOICE
         WHERE RECEIVER_CONTACT = @P1 AND SIRA_NO > @P2
         ORDER BY SIRA_NO ASC",
        dbname // e.g. "uut_24_6" or "uut_25_1"
    );
    let mut query = Query::new(sql_sentence);
    query.bind(source_vkntckn.clone());
    query.bind(after_this);

    let mut conn = pool
        .get()
        .await
        .map_err(DbError::from)
        .ctx("get_incoming_invoice_recs")?;

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(DbError::from)
        .ctx("get_incoming_invoice_recs:Query")?;
    let rows = stream
        .into_first_result()
        .await
        .map_err(DbError::from)
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
