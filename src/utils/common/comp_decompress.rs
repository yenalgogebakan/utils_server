use crate::utils::errors::invoice_conversion_errors::InvConvError;
use std::io::{Cursor, Read};
use std::time::Duration;
use tokio_util::bytes;
use xz2::read::XzDecoder;

pub const DECOMPRESS_ASYNC_THRESHOLD: i32 = 2 * 1024 * 1024; // 2MB
pub async fn xz_decompress(
    content: bytes::Bytes,
    uncompressed_size: usize,
    object_id: &str,
) -> Result<bytes::Bytes, InvConvError> {
    if (uncompressed_size as i32) < DECOMPRESS_ASYNC_THRESHOLD {
        // Small file: decompress inline (sync)
        decompress_sync(content, uncompressed_size).map_err(|e| InvConvError::DecompressError {
            object_id: object_id.to_string(),
            source: e,
        })
    } else {
        // Large file: spawn blocking task with timeout

        let decompression_task =
            tokio::task::spawn_blocking(move || decompress_sync(content, uncompressed_size));

        // Race between completion, timeout, and cancellation
        tokio::select! {
            join_result = decompression_task => {
                match join_result {
                    Ok(decompress_result) => {
                        decompress_result.map_err(|e| InvConvError::DecompressError {
                            object_id : object_id.to_string(),
                            source: e,
                        })
                    }
                    Err(join_err) => {
                        Err(InvConvError::TaskJoinError(join_err.to_string()))
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(30)) => {
                cancellation_token.cancel();
                Err(InvConvError::DecompressTimeout {
                    object_id: object_id.to_string(),
                    timeout_secs: 30,
                })
            }
            _ = token_clone.cancelled() => {
                Err(InvConvError::DecompressCancelled  (object_id.to_string()))
            }
        }
    }
}

fn decompress_sync(
    content: bytes::Bytes,
    uncompressed_size: usize,
) -> std::io::Result<bytes::Bytes> {
    let mut decoder = XzDecoder::new(Cursor::new(content));
    let mut decompressed_vec = Vec::with_capacity(uncompressed_size + 32);
    decoder.read_to_end(&mut decompressed_vec)?;

    assert!(
        decompressed_vec.len() <= uncompressed_size + 32,
        "Decompressed size mismatch: expected ~{}, got {}",
        uncompressed_size,
        decompressed_vec.len()
    );

    Ok(decompressed_vec.into())
}
