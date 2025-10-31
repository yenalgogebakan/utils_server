use crate::utils::errors::doc_processing_errors::{DocProcessingError, ErrCtx};
use crate::utils::object_store::{object_store::Store, opendal_mssql_wrapper::ObjectStoreRecord};

pub async fn get_ubl_decomp_from_object_store(
    object_store: &Store,
    object_id: &str,
    year: &String,
) -> Result<Vec<u8>, DocProcessingError> {
    // Check if the compressed ubl exists
    //     // Check if the compressed ubl exists
    if !object_store.object_exists("ubls", object_id, year).await? {
        return Err(DocProcessingError::UblNotFoundInObjectStore(
            object_id.to_string(),
        ));
    }

    let object_store_rec: ObjectStoreRecord = object_store.get("ubls", object_id, year).await?;

    Ok(Vec::new())

    /*
    // get compressed ubl
    let object_store_rec: ObjectStoreRecord =
        object_store.get("ubls", path_in_object_store, year).await?;
    let object_id_for_error = path_in_object_store.to_string();
    let decompressed = if object_store_rec.original_size >= DECOMPRESS_ASYNC_THRESHOLD {
        // Large file: offload to blocking thread
        let object_id_for_error_clone = object_id_for_error.clone();
        tokio::task::spawn_blocking(move || {
            xz_decompress(
                &object_store_rec.objcontent,
                object_store_rec.original_size as usize,
            )
            .map_err(|e| ProcessError::DecompressError {
                object_id: object_id_for_error_clone,
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
            // Construct the error manually
            object_id: object_id_for_error.clone(), // Use the object_id
            source: e,                              // The io::Error from xz_decompress
        })?
    }; */
}
