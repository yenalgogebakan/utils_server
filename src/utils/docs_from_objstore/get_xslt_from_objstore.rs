use crate::utils::common::comp_decompress::{DECOMPRESS_ASYNC_THRESHOLD, xz_decompress};
use crate::utils::errors::process_errors::ProcessError;
use crate::utils::object_store::{object_store::Store, opendal_mssql_wrapper::ObjectStoreRecord};

pub async fn get_xslt_from_objstore(
    object_store: &Store,
    year: &String,
    xslt_key: &String, // object_id of the xslt
) -> Result<Vec<u8>, ProcessError> {
    // Try compressed first
    let key_xz = format!("{xslt_key}.xz");
    let object_id_for_error = xslt_key.to_string();
    if object_store.object_exists("xslt", &key_xz, year).await? {
        // Compressed version exists
        let object_store_rec = object_store.get("xslt", &key_xz, year).await?;

        let decompressed = if object_store_rec.original_size >= DECOMPRESS_ASYNC_THRESHOLD {
            // Large file: offload to blocking thread
            //let object_id_clone = object_id_for_error.clone();
            tokio::task::spawn_blocking(move || {
                xz_decompress(
                    &object_store_rec.objcontent,
                    object_store_rec.original_size as usize,
                )
                .map_err(|e| ProcessError::DecompressError {
                    object_id: object_id_for_error,
                    source: e,
                })
            })
            .await
            .map_err(ProcessError::TaskJoinError)??
        } else {
            // Small file: decompress inline
            xz_decompress(
                &object_store_rec.objcontent,
                object_store_rec.original_size as usize,
            )
            .map_err(|e| ProcessError::DecompressError {
                object_id: object_id_for_error,
                source: e,
            })?
        };

        Ok(decompressed)
    } else if object_store.object_exists("xslt", xslt_key, year).await? {
        // Fallback: uncompressed version exists
        let object_store_rec = object_store.get("xslt", xslt_key, year).await?;

        // Uncompressed - just return the content directly
        Ok(object_store_rec.objcontent)
    } else {
        // Neither compressed nor uncompressed exists
        Err(ProcessError::XsltNotFoundInObjectStore(object_id_for_error))
    }
}
