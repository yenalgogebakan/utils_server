use crate::utils::errors::object_store_errors::ObjectStoreError;
use crate::utils::object_store::opendal_mssql_wrapper::{MssqlStore, ObjectStoreRecord};

/*
pub enum Store {
    Dal(DalStore),
    Mssql(MssqlStore),
}

impl Store {
    async fn put(&self, bucket: &str, key: &str, data: &[u8]) -> Result<()> {
        match self {
            Store::Dal(s) => s.put(bucket, key, data).await,
            Store::Mssql(s) => s.put(bucket, key, data).await,
        }
    }

    async fn get(&self, bucket: &str, key: &str) -> Result<Vec<u8>> {
        match self {
            Store::Dal(s) => s.get(bucket, key).await,
            Store::Mssql(s) => s.get(bucket, key).await,
        }
    }
}

/// Demo main
#[tokio::main]
async fn main() -> Result<()> {
    // switch backend
    let use_mssql = false;

    let bucket = "demo";
    let key = "hello.txt";
    let data = b"Hello world from Store enum!";

    let store = if use_mssql {
        Store::Mssql(MssqlStore::new_mssql().await?)
    } else {
        Store::Dal(DalStore::new_minio().await?)
    };

    store.put(bucket, key, data).await?;
    let out = store.get(bucket, key).await?;
    println!("Got {} bytes: {}", out.len(), String::from_utf8_lossy(&out));

    Ok(())
}
*/
#[derive(Debug, Clone)]
pub enum Store {
    //Dal(DalStore),
    Mssql(MssqlStore),
}

impl Store {
    /*
    async fn put(&self, bucket: &str, key: &str, data: &[u8]) -> Result<()> {
        match self {
            Store::Dal(s) => s.put(bucket, key, data).await,
            Store::Mssql(s) => s.put(bucket, key, data).await,
        }
    }
    */

    pub async fn get(
        &self,
        bucket: &str,
        key: &str,
        year: &str,
    ) -> Result<ObjectStoreRecord, ObjectStoreError> {
        match self {
            //Store::Dal(s) => s.get(bucket, key).await,
            Store::Mssql(s) => s.get(bucket, key, year).await,
        }
    }
}
