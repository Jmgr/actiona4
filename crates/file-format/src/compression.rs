use std::io;

use zstd::bulk;

// Safety limits for operations that allocate based on untrusted metadata.
// Largest accepted compressed binary payload.
pub const MAX_BINARY_COMPRESSED_SIZE: usize = 256 * 1024 * 1024;
// Largest accepted binary payload after zstd decompression.
pub const MAX_BINARY_UNCOMPRESSED_SIZE: usize = 1024 * 1024 * 1024;
// Largest accepted JSON attachment after zstd decompression.
pub const MAX_ATTACHMENT_UNCOMPRESSED_SIZE: usize = 512 * 1024 * 1024;
// Largest accepted JSON file on disk, before its Base64 payloads are decoded.
pub const MAX_JSON_FILE_SIZE: usize = 2 * 1024 * 1024 * 1024;
// Largest PCM buffer accepted while normalizing an audio attachment.
pub const MAX_AUDIO_DECODED_SIZE: usize = 128 * 1024 * 1024;
// Small JSON values skip compression work; compression must still reduce Base64 size.
pub const JSON_COMPRESSION_THRESHOLD: usize = 4 * 1024;

pub fn compress(data: &[u8]) -> io::Result<Vec<u8>> {
    bulk::compress(data, 3)
}

// Enforces both a caller-provided limit and the exact size recorded in the file format.
pub fn decompress(data: &[u8], expected_size: usize, limit: usize) -> io::Result<Vec<u8>> {
    if expected_size > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "decompressed payload exceeds the configured size limit",
        ));
    }

    let decoded = bulk::decompress(data, expected_size)?;
    if decoded.len() != expected_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "decompressed payload has an unexpected length",
        ));
    }

    Ok(decoded)
}
