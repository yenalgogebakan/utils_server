//use anyhow::Result;
use super::s3object::S3Object;
//use std::collections::HashMap;

pub trait ObjectStore {
    /// Retrieves data from the object store
    ///
    /// # Arguments
    /// * `year` - The year partition
    /// * `bucket` - The bucket name
    /// * `object_id` - The object identifier
    ///
    /// # Returns
    /// Result containing the data as a Vec<u8> or an error
    //fn get_data(&self, year: i32, bucket: &str, object_id: &str) -> anyhow::Result<Vec<u8>> {
    //    Err(anyhow::anyhow!("Not implemented"))
    //}

    /// Deletes data from the object store
    ///
    /// # Arguments
    /// * `year` - The year partition
    /// * `bucket` - The bucket name
    /// * `object_id` - The object identifier
    ///
    /// # Returns
    /// Result containing the number of objects deleted or an error
    //fn delete_data(&self, year: i32, bucket: &str, object_id: &str) -> anyhow::Result<i32> {
    //    Err(anyhow::anyhow!("Not implemented"))
    //}

    /// Retrieves metadata for an object
    ///
    /// # Arguments
    /// * `year` - The year partition
    /// * `bucket` - The bucket name
    /// * `object_id` - The object identifier
    ///
    /// # Returns
    /// HashMap containing metadata key-value pairs
    //fn get_metadata(
    //    &self,
    //    year: i32,
    //    bucket: &str,
    //    object_id: &str,
    //) -> anyhow::Result<HashMap<String, String>> {
    //    Err(anyhow::anyhow!("Not implemented"))
    //}

    /// Checks if an object exists
    ///
    /// # Arguments
    /// * `year` - The year partition
    /// * `bucket` - The bucket name
    /// * `object_id` - The object identifier
    ///
    /// # Returns
    /// true if the object exists, false otherwise
    //fn object_exists(&self, year: i32, bucket: &str, object_id: &str) -> anyhow::Result<bool> {
    //    Err(anyhow::anyhow!("Not implemented"))
    //}

    /// Stores data in the object store
    ///
    /// # Arguments
    /// * `year` - The year partition
    /// * `bucket` - The bucket name
    /// * `object_id` - The object identifier
    /// * `original_size` - The original size of the content
    /// * `content` - The data to store
    /// * `user_metadata` - User-defined metadata
    ///
    /// # Returns
    /// true if successful, false otherwise

    //fn put_data(
    //    &self,
    //    year: i32,
    //    bucket: &str,
    //    object_id: &str,
    //    original_size: u64,
    //    content: Vec<u8>,
    //    user_metadata: HashMap<String, String>,
    //) -> anyhow::Result<()> {
    //    Err(anyhow::anyhow!("Not implemented"))
    //}

    /*****/
    fn new_with_yearpath(year: i32, path: &str) -> Self;
    fn get_dbname(&self) -> anyhow::Result<String>;

    fn get_ubl(&self) -> anyhow::Result<S3Object>;
}
