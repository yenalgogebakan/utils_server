use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use tokio_util::bytes;
use zip::result::ZipError;
use zip::{ZipWriter, write::FileOptions};

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

    pub fn close_zip(mut self) -> Result<Vec<u8>, ZipError> {
        // 1. Apply the final Zip footer (Central Directory).
        // This consumes the ZipWriter and returns the underlying File.
        let mut file = self.zip.finish()?;

        // 2. The file cursor is currently at the end of the file.
        // We need to rewind it to the beginning to read the data back.
        file.seek(SeekFrom::Start(0))?;

        // 3. Optimization: Get the file size to pre-allocate the Vec.
        // This ensures we only allocate memory once, preventing multiple
        // resizing operations as the buffer grows.
        let len = file.metadata()?.len() as usize;
        let mut buffer = Vec::with_capacity(len);

        // 4. Read the file contents into the buffer.
        file.read_to_end(&mut buffer)?;

        Ok(buffer)
    }
}
