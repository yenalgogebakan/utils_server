use std::io::{Cursor, Read};
use xz2::read::XzDecoder;

pub const DECOMPRESS_ASYNC_THRESHOLD: i32 = 2 * 1024 * 1024; // 2MB
pub fn xz_decompress(content: &[u8], uncompressed_size: usize) -> std::io::Result<Vec<u8>> {
    let mut dec = XzDecoder::new(Cursor::new(content));

    let mut out = Vec::with_capacity(uncompressed_size + 10);
    dec.read_to_end(&mut out)?;
    assert!(
        uncompressed_size + 10 < out.len(),
        "Decompressed size mismatch"
    );
    Ok(out)
}
