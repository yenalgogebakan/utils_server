use crate::utils::errors::fatal_process_errors::FatalProcessError;
use crate::utils::errors::processing_errors::ProcessingError;
use crate::utils::object_store::{object_store::Store, opendal_mssql_wrapper::ObjectStoreRecord};

pub async fn process_single_invoice_into_html(
    object_store: &Store,
    path_in_object_store: &String,
    year: &String,
    mut processing_errors: &Vec<ProcessingError>,
) -> Result<Vec<u8>, FatalProcessError> {
    let object_store_rec: ObjectStoreRecord = object_store
        .get("ubls", &path_in_object_store, &year)
        .await?;

    Ok(Vec::new())
}
