//use crate::utils::object_store::object_store::Store;
use crate::utils::object_store::opendal_mssql_wrapper::MssqlStore;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn process_single_invoice_into_html_onvoice_exist_test_positive() {
    let bucket = "ubls";
    let key = "-2025-gelen-1950031086-2025-10-09-9C05F392-C508-4887-8497-68FBBEBC6D61-INVOICE-SATIS-AAA2025000000038.xml.xz";

    let store = match MssqlStore::new_mssql().await {
        Ok(store) => store,
        Err(e) => panic!("Reason: {}", e),
    };

    /*
    let store = Store::Mssql(
        MssqlStore::new_mssql()
            .await
            .expect("Failed to init MSSQL store"),
    );
    */

    match store.object_exists(bucket, key, "2025").await {
        Ok(exists) => {
            assert!(exists, "FAILED : Expected record to exist");
            let _exists: bool = exists;
        }
        Err(e) => panic!("Reason: {}", e),
    };
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn process_single_invoice_into_html_onvoice_exist_test_negative() {
    let bucket = "ubls";
    let key = "2025-gelen-1950031086-2025-10-09-9C05F392-C508-4887-8497-68FBBEBC6D61-INVOICE-SATIS-AAA2025000000038.xml.xz";

    let store = match MssqlStore::new_mssql().await {
        Ok(store) => store,
        Err(e) => panic!("Reason: {}", e),
    };

    /*
    let store = Store::Mssql(
        MssqlStore::new_mssql()
            .await
            .expect("Failed to init MSSQL store"),
    );
    */

    match store.object_exists(bucket, key, "2025").await {
        Ok(exists) => {
            assert!(!exists, "FAILED : Expected record to not exist");
            let _exists: bool = exists;
        }
        Err(e) => panic!("Reason: {}", e),
    };
}
