use crate::utils::database_manager::init_database::{DbPools, init_db_connection_pools};
use crate::utils::download_request::download_request_types::{
    DownloadDocRequest, DownloadFormat, DownloadType,
};
use crate::utils::get_and_process_invoices::process_invoices_types::GetAndProcessInvoicesRequest;
use crate::utils::object_store::object_store::Store;
use crate::utils::object_store::opendal_mssql_wrapper::MssqlStore;

use std::sync::Arc;

async fn get_new_pools() -> anyhow::Result<DbPools> {
    let db_pools: DbPools = init_db_connection_pools().await?;
    Ok(db_pools)
}

#[tokio::test]
async fn test_no_invoice_after_sirano() {
    let pools: DbPools = get_new_pools().await.unwrap();

    let request = DownloadDocRequest {
        source_vkntckn: "1950031086".to_string(),
        after_this: 26000,
        download_type: DownloadType::Html,
        format: DownloadFormat::Zip,
    };

    let invoices = request
        .get_incoming_invoice_recs(&pools.incoming_invoice_pool)
        .await
        .unwrap();
    assert_eq!(invoices.len(), 0);
}

#[tokio::test]
async fn test_get_100_invoices() {
    let pools: DbPools = get_new_pools().await.unwrap();

    let request = DownloadDocRequest {
        source_vkntckn: "1950031086".to_string(),
        after_this: 24000,
        download_type: DownloadType::Html,
        format: DownloadFormat::Zip,
    };

    let invoices = request
        .get_incoming_invoice_recs(&pools.incoming_invoice_pool)
        .await
        .unwrap();
    assert!(invoices.len() == 100, "Should retrieve 100 invoices");
}

#[tokio::test]
async fn test_get_44_invoices() {
    let pools: DbPools = get_new_pools().await.unwrap();

    let request = DownloadDocRequest {
        source_vkntckn: "1950031086".to_string(),
        after_this: 25000,
        download_type: DownloadType::Html,
        format: DownloadFormat::Zip,
    };
    let request: Arc<DownloadDocRequest> = Arc::new(request);

    let invoices = request
        .get_incoming_invoice_recs(&pools.incoming_invoice_pool)
        .await
        .unwrap();
    assert!(invoices.len() == 9, "Should retrieve 44 invoices");

    println!("✅ Found {} invoice records", invoices.len());
    let process_invoices_req = GetAndProcessInvoicesRequest {
        request: request.clone(),
        invoices: invoices,
    };
    let object_store = Store::Mssql(
        MssqlStore::new_mssql()
            .await
            .expect("Failed to init MSSQL store"),
    );
    match process_invoices_req.process(&object_store).await {
        Ok(result) => {
            println!(
                "✅ Processed {} invoices, size: {} bytes",
                result.record_count, result.size_bytes
            );
        }
        Err(e) => {
            assert!(4 == 5, "❌ Processing error: {}", e);
        }
    }
}

#[tokio::test]
async fn test_invoices_sorted() {
    let pools: DbPools = get_new_pools().await.unwrap();

    let request = DownloadDocRequest {
        source_vkntckn: "1950031086".to_string(),
        after_this: 24000,
        download_type: DownloadType::Html,
        format: DownloadFormat::Zip,
    };

    let invoices = request
        .get_incoming_invoice_recs(&pools.incoming_invoice_pool)
        .await
        .unwrap();

    for i in 1..invoices.len() {
        assert!(
            invoices[i].sira_no >= invoices[i - 1].sira_no,
            "Invoices should be sorted ascending"
        );
    }
}
