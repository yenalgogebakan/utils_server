use crate::utils::object_store::object_store::Store;
use crate::utils::object_store::opendal_mssql_wrapper::MssqlStore;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_mssql_store_put_get() {
    let bucket = "ubls";
    let key = "-2025-gelen-1950031086-2025-10-09-9C05F392-C508-4887-8497-68FBBEBC6D61-INVOICE-SATIS-AAA2025000000038.xml.xz";

    //let store = match Store::Mssql(MssqlStore::new_mssql().await) {
    //    Store::Mssql(s) => s,
    //    _ => panic!("Expected MssqlStore"),
    //};
    let store = Store::Mssql(
        MssqlStore::new_mssql()
            .await
            .expect("Failed to init MSSQL store"),
    );
    let data = store
        .get(bucket, key, "2025")
        .await
        .expect("Failed to get object from MSSQL store");
    println!("Got {} bytes", data.objcontent.len());
}
