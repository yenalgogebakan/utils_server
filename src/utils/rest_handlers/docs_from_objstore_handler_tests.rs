use crate::utils::appstate::appstate::{AppState, create_app};
use crate::utils::database_manager;
use crate::utils::object_store::object_store::Store;
use crate::utils::object_store::opendal_mssql_wrapper::MssqlStore;

use crate::utils::common::download_types_and_formats::{
    DownloadFormat, DownloadType, FilenameInZipMode,
};
use crate::utils::rest_handlers::docs_from_objstore_handler::{
    DocRequestItem, DocsFromObjStoreReq,
};
use axum_test::TestServer;
use std::sync::Arc;

#[tokio::test]
async fn test2() {
    let db_pools = match database_manager::init_database::init_db_connection_pools().await {
        Ok(pools) => {
            println!("✅ All DB pools initialized successfully.");
            pools
        }

        Err(err) => {
            eprintln!("❌ Database initialization failed: {err}");
            //for cause in anyhow::Chain::new(&err) {
            //    eprintln!("  caused by: {cause}");
            //}
            std::process::exit(1);
        }
    };
    let object_store = Store::Mssql(
        MssqlStore::new_mssql()
            .await
            .expect("Failed to init MSSQL store"),
    );

    let app_state = Arc::new(AppState {
        db_pools,
        object_store,
    });

    let server = TestServer::new(create_app(app_state)).unwrap();

    let request_body = DocsFromObjStoreReq {
        target_type: DownloadType::Html,
        target_format: DownloadFormat::Zip,
        year: "2025".to_string(),
        filename_in_zip: FilenameInZipMode::StartFromInvoiceOne,
        items: vec![
            DocRequestItem {
                object_id: "2025-gelen-1950031086-2025-10-09-9C05F392-C508-4887-8497-68FBBEBC6D61-INVOICE-SATIS-AAA2025000000038.xml.xz".to_string(),
                sira_no: Some(1),
                invoice_no: Some("INV-001".to_string()),
            },
            DocRequestItem {
                object_id: "2025-gelen-1950031086-2025-10-09-9C05F392-C508-4887-8497-68FBBEBC6D61-INVOICE-SATIS-AAA2025000000038.xml.xz".to_string(),
                sira_no: Some(2),
                invoice_no: Some("INV-002".to_string()),
            },
        ],
    };

    let response = server
        .get("/api/v1/docs_from_objstore") // Note: POST request and full path
        .json(&request_body) // Send JSON body
        .await;

    // 5. Assertions

    // First, assert the status code
    response.assert_status_ok(); // Assert status is 200 OK

    /*
    // Then, assert the JSON body
    let json_response: DocsFromObjStoreResponse = response.json(); // Deserialize response body

    // Now you can assert individual fields of the response struct
    assert_eq!(json_response.docs_count, 2);
    assert_eq!(json_response.size, 1024);
    assert_eq!(json_response.request_fully_completed, true);
    assert_eq!(json_response.last_processed_sira_no, Some(12345));
    // You might also want to assert the content of `data` if it's predictable
    assert_eq!(json_response.data, b"some_base64_encoded_zip_data".to_vec());
    */
}
