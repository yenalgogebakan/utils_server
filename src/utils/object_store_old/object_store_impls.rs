use super::object_store::ObjectStore;
use super::s3object::S3Object;

impl ObjectStore for S3Object {
    fn get_ubl(&self) -> anyhow::Result<S3Object> {
        self.do_getubl()
    }

    fn new_with_yearpath(year: i32, path: &str) -> Self {
        let sanitized = format!("{}.xz", path.replace("/", "-"));

        Self {
            year,
            path: path.to_string(),
            path_sanitized: sanitized,
            bucket: "".to_string(),
            only_ubl: Vec::new(),
            ubl: Vec::new(),
            full_ubl: Vec::new(),

            xslt: None,
            cert: None,
        }
    }

    fn get_dbname(&self) -> anyhow::Result<String> {
        if self.bucket.is_empty() {
            return Err(anyhow::anyhow!("Bucket is empty"));
        }

        Ok("s3object_db".to_string())
    }
}
