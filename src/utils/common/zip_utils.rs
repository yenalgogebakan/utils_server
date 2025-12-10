use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use tempfile::tempfile;
use tokio_util::bytes;
use zip::{CompressionMethod, ZipWriter, write::FileOptions};

pub struct ZipFile {
    pub zip: ZipWriter<std::fs::File>,
    pub zip_opts: zip::write::FileOptions,
}

impl ZipFile {
    // Return a Result instead of just the struct
    pub fn new() -> io::Result<Self> {
        // Use '?' to return the error immediately if it fails
        let tmp_file = tempfile::tempfile()?;
        let zip = ZipWriter::new(tmp_file);
        let zip_opts = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        Ok(ZipFile { zip, zip_opts })
    }

    pub fn write_to_zip(&mut self, filename: &str, content: bytes::Bytes) -> io::Result<()> {
        // Start a new file inside the zip
        self.zip.start_file(filename, self.zip_opts)?;

        // Write the bytes
        self.zip.write_all(&content)?;

        Ok(())
    }
}
