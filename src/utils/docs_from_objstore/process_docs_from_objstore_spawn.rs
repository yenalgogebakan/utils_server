use tokio::sync::OwnedSemaphorePermit;

use crate::utils::appstate::appstate::SharedState;
use crate::utils::errors::doc_processing_errors::DocProcessingError;
use crate::utils::rest_handlers::docs_from_objstore_spawn_handler::{
    DocsFromObjStoreReq, DocsFromObjStoreResponse,
};

pub fn process_docs_from_objstore_spawn(
    req: DocsFromObjStoreReq,
    state: SharedState,
    _permit: OwnedSemaphorePermit, // keep as parameter so it's dropped when this fn returns
) -> Result<DocsFromObjStoreResponse, DocProcessingError> {
    // We are in the sync thread, in which we will read ubls and convert them to HTML one by one

    
    Ok(DocsFromObjStoreResponse {
        data: Vec::new(),
        docs_count: req.items.len() as u32,
        size: 0,
        last_processed_sira_no: req.items.iter().filter_map(|i| i.sira_no).max(),
        request_fully_completed: true,
    })
}
