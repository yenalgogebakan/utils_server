use opendal::Operator;
use opendal::layers::LoggingLayer;
use opendal::services::S3;

use crate::utils::errors::object_store_errors::ObjectStoreError;

// OpenDaL wrapper for MinIO
pub struct DalStore {
    op: Operator,
}
impl DalStore {
    pub async fn new_minio() -> Result<Self, ObjectStoreError> {
        let b = S3::default()
            .bucket("my-bucket")
            .endpoint("http://127.0.0.1:9000")
            .region("us-east-1")
            .access_key_id("minioadmin")
            .secret_access_key("minioadmin");

        let op = Operator::new(b)?.finish().layer(LoggingLayer::default());
        Ok(Self { op })
    }

    pub async fn put(
        &self,
        bucket: &str,
        key: &str,
        year: &str,
        data: Vec<u8>,
    ) -> Result<(), ObjectStoreError> {
        let path = format!("{}/{}", bucket, key);
        self.op.write(&path, data).await?;
        Ok(())
    }

    pub async fn get(
        &self,
        bucket: &str,
        key: &str,
        year: &str,
    ) -> Result<Vec<u8>, ObjectStoreError> {
        let path = format!("{}/{}", bucket, key);
        let buf = self.op.read(&path).await?;
        Ok(buf.to_vec())
    }
}
