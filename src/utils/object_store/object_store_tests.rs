use crate::utils::errors::object_store_errors::ObjectStoreError;
use crate::utils::object_store::object_store::Store;
use crate::utils::object_store::opendal_mssql_wrapper::MssqlStore;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_mssql_store_put_get() {
    let bucket = "demo";
    let key = "hello.txt";
    let data = b"Hello world from MssqlStore!";

    let store = match Store::Mssql(MssqlStore::new_mssql().await?) {
        Store::Mssql(s) => s,
        //_ => panic!("Expected MssqlStore"),
    };

    let out = store.get(bucket, key).await.unwrap();
    println!("Got {} bytes: {}", out.len(), String::from_utf8_lossy(&out));

    assert!(out == data, "Data retrieved should match data stored");

    // Test error handling for missing key
    let missing_key = "nonexistent.txt";
    match store.get(bucket, missing_key).await {
        Ok(_) => panic!("Expected error for missing key, but got data"),
        Err(e) => match e {
            ObjectStoreError::OpenDALError(_) => {
                println!("Correctly received error for missing key: {}", e)
            }
            _ => panic!("Unexpected error type: {}", e),
        },
    }
}
