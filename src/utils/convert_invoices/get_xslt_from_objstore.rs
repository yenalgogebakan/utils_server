use crate::utils::common::comp_decompress::xz_decompress;
use crate::utils::errors::invoice_conversion_errors::{ErrCtx, InvConvError};
use crate::utils::errors::object_store_errors::ObjectStoreError;
use crate::utils::object_store::object_store::Store;
use tokio_util::bytes;

pub async fn get_xslt_from_objstore(
    object_store: &Store,
    year: &String,
    xslt_key: &String, // object_id of the xslt
) -> Result<bytes::Bytes, InvConvError> {
    // Try compressed first
    let xslt_key_xz = format!("{xslt_key}.xz");
    let object_store_rec_for_xslt = match object_store.get("xslts", &xslt_key_xz, year).await {
        Ok(rec) => rec, // <— found compressed
        Err(ObjectStoreError::NoRecordFound(..)) => {
            // Not found compressed, will try uncompressed next
            match object_store.get("xslts", xslt_key, year).await {
                Ok(rec) => rec, // Found uncompressed
                Err(err) => {
                    let _err: InvConvError = err.into();
                    return Err(_err)
                        .ctx("get_xslt_from_objstore : xslt not found compressed or uncompressed");
                }
            }
        }
        Err(err) => return Err(err.into()),
    };

    let uncompressed_size = object_store_rec_for_xslt.original_size as usize;
    let decompressed: bytes::Bytes = match xz_decompress(
        object_store_rec_for_xslt.objcontent,
        uncompressed_size,
        xslt_key,
    )
    .await
    {
        Ok(decompressed) => decompressed,
        Err(err) => {
            let _err: InvConvError = err.into();
            return Err(_err).ctx("get_xslt_from_objstore : xz_decompress failed");
        }
    };

    Ok(decompressed)
}
