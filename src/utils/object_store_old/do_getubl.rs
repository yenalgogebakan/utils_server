use crate::utils::object_store_old::s3object::S3Object;

impl S3Object {
    pub fn do_getubl(&self) -> anyhow::Result<S3Object> {
        println!("in do_getubl S3Object: {}", self);

        Ok(S3Object::default())
    }
}
