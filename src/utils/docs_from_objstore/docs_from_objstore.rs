use crate::utils::common::build_zip::{build_tzip, build_zip};
use crate::utils::common::common_types_and_formats::XsltCache;
use crate::utils::common::download_types_and_formats::{
    DownloadFormat, DownloadType, FilenameInZipMode,
};
use crate::utils::errors::processing_errors::ProcessError;
use crate::utils::object_store::object_store::Store;
use crate::utils::rest_handlers::docs_from_objstore_handler::{
    DocRequestItem, DocsFromObjStoreReq,
};

const HTML_BYTES_LIMIT: u64 = 1024 * 1024;

#[derive(Debug, Clone)]
pub struct DocFromObjStoreItem {
    pub object_id: String,
    pub sira_no: Option<i64>,
    pub invoice_no: Option<String>,
}
impl From<DocRequestItem> for DocFromObjStoreItem {
    fn from(req_item: DocRequestItem) -> Self {
        DocFromObjStoreItem {
            object_id: req_item.object_id,
            sira_no: req_item.sira_no,
            invoice_no: req_item.invoice_no,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DocsFromObjStore {
    pub target_type: DownloadType,
    pub target_format: DownloadFormat,
    pub year: String,
    pub filename_in_zip: FilenameInZipMode,
    pub last_processed_sira_no: Option<i64>,

    /// Items to fetch/process
    pub items: Vec<DocFromObjStoreItem>,
}
impl From<DocsFromObjStoreReq> for DocsFromObjStore {
    fn from(req: DocsFromObjStoreReq) -> Self {
        DocsFromObjStore {
            target_type: req.target_type,
            target_format: req.target_format,
            year: req.year,
            filename_in_zip: req.filename_in_zip,
            last_processed_sira_no: None, // This field is not in the request, so initialize to None or a sensible default
            items: req
                .items
                .into_iter()
                .map(DocFromObjStoreItem::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DocsFromObjStoreResult {
    pub data: Vec<u8>,
    pub docs_count: u32,
    pub size: u64,
    pub last_processed_sira_no: Option<i64>,
}

impl DocsFromObjStore {
    pub async fn do_process(
        &self,
        object_store: &Store,
    ) -> Result<DocsFromObjStoreResult, ProcessError> {
        let mut processed: Vec<(String, Vec<u8>)> = Vec::new();
        let mut last_processed_sira_no = 0;
        let mut total_html_bytes = 0u64;
        let mut docs_count = 0u32;

        let mut xlst_cache: XsltCache;

        for item in &self.items {
            match process_single_invoice_into_html(object_store, &self.year, &item.object_id).await
            {
                Ok(html) => {
                    let len = html.len() as u64;
                    if total_html_bytes + len > HTML_BYTES_LIMIT {
                        break;
                    }
                    processed.push((self.filename_in_zip(item, docs_count), html));
                    total_html_bytes += len;
                    docs_count += 1;
                    last_processed_sira_no = item.sira_no.unwrap_or_default();
                }
                Err(e) => match e {
                    ProcessError::UblNotFoundInObjectStore(_) => {
                        let data = match self.target_format {
                            DownloadFormat::Zip => build_zip(processed)?,
                            DownloadFormat::TZip => build_tzip(processed)?,
                            DownloadFormat::Gzip => build_zip(processed)?,
                        };
                        return Ok(DocsFromObjStoreResult {
                            data,
                            docs_count,
                            size: total_html_bytes,
                            last_processed_sira_no: Some(last_processed_sira_no),
                        });
                    }
                    _ => {
                        return Ok(DocsFromObjStoreResult::default());
                    }
                },
            };
        }

        let data = match self.target_format {
            DownloadFormat::Zip => build_zip(processed)?,
            DownloadFormat::TZip => build_tzip(processed)?,
            DownloadFormat::Gzip => build_zip(processed)?,
        };
        Ok(DocsFromObjStoreResult {
            data,
            docs_count,
            size: total_html_bytes,
            last_processed_sira_no: Some(last_processed_sira_no),
        })
    }

    fn filename_in_zip(&self, item: &DocFromObjStoreItem, docs_count: u32) -> String {
        match self.filename_in_zip {
            FilenameInZipMode::ExtractFromObjID => "Invoice_001".to_string(),
            FilenameInZipMode::IncludedInRequest => match &item.invoice_no {
                Some(value) => format!("Fat_{}", value),
                None => format!("Fat_00000000"),
            },
            FilenameInZipMode::UseSiraNo => match item.sira_no {
                Some(value) => format!("Fat_{}", value),
                None => format!("Fat_00000000"),
            },
            FilenameInZipMode::StartFromInvoiceOne => format!("Fat_{}", docs_count),
        }
    }
}

pub async fn process_single_invoice_into_html(
    object_store: &Store,
    year: &String,
    path_in_object_store: &String, //key
) -> Result<Vec<u8>, ProcessError> {
    if !object_store
        .object_exists("ubls", path_in_object_store, year)
        .await?
    {
        return Err(ProcessError::UblNotFoundInObjectStore(
            path_in_object_store.to_string(),
        ));
    }

    Ok(Vec::new())
}
