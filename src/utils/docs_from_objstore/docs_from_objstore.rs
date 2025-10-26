use flate2::Compression;
use flate2::write::GzEncoder;
use std::collections::HashMap;
use std::io::{Cursor, Write};
use tar::Builder as TarBuilder;
use xz2::write::XzEncoder;
use zip::{CompressionMethod, ZipWriter, write::FileOptions};

use crate::utils::common::common_types_and_formats::{Xslt, XsltCache};
use crate::utils::common::comp_decompress::{DECOMPRESS_ASYNC_THRESHOLD, xz_decompress};
use crate::utils::common::download_types_and_formats::{
    DownloadFormat, DownloadType, FilenameInZipMode,
};
use crate::utils::common::san_desanitize::sanitize_fast;

use crate::utils::docs_from_objstore::{
    extract_xslt_key_from_xml::extract_xslt_key_from_xml,
    get_xslt_from_objstore::get_xslt_from_objstore,
};
use crate::utils::errors::process_errors::ProcessError;
use crate::utils::object_store::{object_store::Store, opendal_mssql_wrapper::ObjectStoreRecord};
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
    pub request_fully_completed: bool,
}

impl DocsFromObjStore {
    pub async fn do_process(
        &self,
        object_store: &Store,
    ) -> Result<DocsFromObjStoreResult, ProcessError> {
        let mut last_processed_sira_no = 0;
        let mut total_html_bytes = 0u64;
        let mut docs_count = 0u32;

        let mut xlst_cache: XsltCache = HashMap::with_capacity(4);

        match self.target_format {
            DownloadFormat::Zip => {
                let (mut zip, file_opts) = start_zip();
                for item in &self.items {
                    match process_single_invoice_into_html(
                        object_store,
                        &self.year,
                        &item.object_id,
                        &mut xlst_cache,
                    )
                    .await
                    {
                        Ok(html) => {
                            let len = html.len() as u64;
                            if total_html_bytes + len > HTML_BYTES_LIMIT {
                                break;
                            }
                            zip.start_file(self.filename_in_zip(item, docs_count), file_opts)?;
                            zip.write_all(&html)?;
                            total_html_bytes += len;
                            docs_count += 1;
                            last_processed_sira_no = item.sira_no.unwrap_or_default();
                        }
                        Err(e) => match e {
                            ProcessError::UblNotFoundInObjectStore(_) => {
                                let data = finish_zip(zip).map_err(ProcessError::from)?;
                                let size = data.len() as u64; // payload size
                                return Ok(DocsFromObjStoreResult {
                                    data,
                                    docs_count,
                                    size: size,
                                    last_processed_sira_no: Some(last_processed_sira_no),
                                    request_fully_completed: false,
                                });
                            }
                            _ => {
                                return Ok(DocsFromObjStoreResult::default());
                            }
                        },
                    };
                }
                let data = finish_zip(zip).map_err(ProcessError::from)?;
                let size = data.len() as u64; // payload size
                return Ok(DocsFromObjStoreResult {
                    data,
                    docs_count,
                    size: size,
                    last_processed_sira_no: Some(last_processed_sira_no),
                    request_fully_completed: true,
                });
            }
            DownloadFormat::Tzip => {
                let mut tar = start_tzip();
                for item in &self.items {
                    match process_single_invoice_into_html(
                        object_store,
                        &self.year,
                        &item.object_id,
                        &mut xlst_cache,
                    )
                    .await
                    {
                        Ok(html) => {
                            let len = html.len() as u64;
                            if total_html_bytes + len > HTML_BYTES_LIMIT {
                                break;
                            }
                            let mut header = tar::Header::new_gnu();
                            header.set_mode(0o644);
                            header.set_size(len);
                            header.set_cksum();
                            tar.append_data(
                                &mut header,
                                self.filename_in_zip(item, docs_count),
                                html.as_slice(),
                            )
                            .map_err(ProcessError::from)?;
                            total_html_bytes += len;
                            docs_count += 1;
                            last_processed_sira_no = item.sira_no.unwrap_or_default();
                        }
                        Err(e) => match e {
                            ProcessError::UblNotFoundInObjectStore(_) => {
                                let data = finish_tzip(tar).map_err(ProcessError::from)?;
                                let size = data.len() as u64;
                                return Ok(DocsFromObjStoreResult {
                                    data,
                                    docs_count,
                                    size: size,
                                    last_processed_sira_no: Some(last_processed_sira_no),
                                    request_fully_completed: false,
                                });
                            }
                            _ => {
                                return Ok(DocsFromObjStoreResult::default());
                            }
                        },
                    };
                }
                let data = finish_tzip(tar).map_err(ProcessError::from)?;
                let size = data.len() as u64;
                return Ok(DocsFromObjStoreResult {
                    data,
                    docs_count,
                    size: size,
                    last_processed_sira_no: Some(last_processed_sira_no),
                    request_fully_completed: false,
                });
            }
            DownloadFormat::Gzip => {
                let mut tar = start_targz();
                for item in &self.items {
                    match process_single_invoice_into_html(
                        object_store,
                        &self.year,
                        &item.object_id,
                        &mut xlst_cache,
                    )
                    .await
                    {
                        Ok(html) => {
                            let len = html.len() as u64;
                            if total_html_bytes + len > HTML_BYTES_LIMIT {
                                break;
                            }
                            let mut header = tar::Header::new_gnu();
                            header.set_mode(0o644);
                            header.set_size(len);
                            header.set_cksum();
                            tar.append_data(
                                &mut header,
                                self.filename_in_zip(item, docs_count),
                                html.as_slice(),
                            )
                            .map_err(ProcessError::from)?;
                            total_html_bytes += len;
                            docs_count += 1;
                            last_processed_sira_no = item.sira_no.unwrap_or_default();
                        }
                        Err(e) => match e {
                            ProcessError::UblNotFoundInObjectStore(_) => {
                                let data = finish_targz(tar).map_err(ProcessError::from)?;
                                let size = data.len() as u64;
                                return Ok(DocsFromObjStoreResult {
                                    data,
                                    docs_count,
                                    size: size,
                                    last_processed_sira_no: Some(last_processed_sira_no),
                                    request_fully_completed: false,
                                });
                            }
                            _ => {
                                return Ok(DocsFromObjStoreResult::default());
                            }
                        },
                    };
                }
                let data = finish_targz(tar).map_err(ProcessError::from)?;
                let size = data.len() as u64;
                return Ok(DocsFromObjStoreResult {
                    data,
                    docs_count,
                    size: size,
                    last_processed_sira_no: Some(last_processed_sira_no),
                    request_fully_completed: false,
                });
            }
        }
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
    path_in_object_store: &String, // object_id
    xslt_cache: &mut XsltCache,
) -> Result<Vec<u8>, ProcessError> {
    print!("in process Single invoice into html");

    // Check if the compressed ubl exists
    if !object_store
        .object_exists("ubls", path_in_object_store, year)
        .await?
    {
        return Err(ProcessError::UblNotFoundInObjectStore(
            path_in_object_store.to_string(),
        ));
    }
    // get compressed ubl
    let object_store_rec: ObjectStoreRecord =
        object_store.get("ubls", path_in_object_store, year).await?;
    let object_id_for_error = path_in_object_store.to_string();
    let decompressed = if object_store_rec.original_size >= DECOMPRESS_ASYNC_THRESHOLD {
        // Large file: offload to blocking thread
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
            // Construct the error manually
            object_id: object_id_for_error.to_string(), // Use the object_id
            source: e,                                  // The io::Error from xz_decompress
        })?
    };

    let sanitized = sanitize_fast(&decompressed).map_err(|e| ProcessError::NonUtfCharError {
        object_id: path_in_object_store.to_string(), // Use the object_id
        source: e,                                   // The io::Error from xz_decompress
    })?;

    // we got xml
    let xml: &str = std::str::from_utf8(sanitized.as_ref())
        .expect("Sanitized data should be valid UTF-8 after passing through sanitize_fast");

    let xslt_key = extract_xslt_key_from_xml(xml, &path_in_object_store)?;
    // Look for cache if it already exists, if not get xslt from object store
    if !xslt_cache.contains_key(&xslt_key) {
        println!("Cache miss!");
        let xslt_str = get_xslt_from_objstore(object_store, year, &xslt_key).await?;
        xslt_cache.insert(
            xslt_key.clone(),
            Xslt {
                xslt: xslt_str,
                compiled_xslt: None,
            },
        );
    }
    let xslt = xslt_cache.get(&xslt_key);

    Ok(Vec::new())
}

fn start_zip() -> (ZipWriter<Cursor<Vec<u8>>>, FileOptions) {
    let cursor = Cursor::new(Vec::new());
    let zip = ZipWriter::new(cursor);
    let opts = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);
    (zip, opts)
}
fn finish_zip(mut zip: ZipWriter<Cursor<Vec<u8>>>) -> std::io::Result<Vec<u8>> {
    Ok(zip.finish()?.into_inner())
}

fn start_tzip() -> TarBuilder<XzEncoder<Cursor<Vec<u8>>>> {
    let cursor = Cursor::new(Vec::new());
    let xz = XzEncoder::new(cursor, 6);
    TarBuilder::new(xz)
}
fn finish_tzip(tar: TarBuilder<XzEncoder<Cursor<Vec<u8>>>>) -> std::io::Result<Vec<u8>> {
    let xz = tar.into_inner()?;
    let cursor = xz.finish()?;
    Ok(cursor.into_inner())
}

fn start_targz() -> TarBuilder<GzEncoder<Cursor<Vec<u8>>>> {
    let cursor = Cursor::new(Vec::new());
    let gz = GzEncoder::new(cursor, Compression::default());
    TarBuilder::new(gz)
}

fn finish_targz(tar: TarBuilder<GzEncoder<Cursor<Vec<u8>>>>) -> std::io::Result<Vec<u8>> {
    let gz = tar.into_inner()?;
    let cursor = gz.finish()?;
    Ok(cursor.into_inner())
}
