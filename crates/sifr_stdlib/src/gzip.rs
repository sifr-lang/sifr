use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use std::io::{Read, Write};

pub fn gzip_compress_bytes(data: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    // GzEncoder<Vec<u8>> has no external IO sink; preserve the historical
    // non-Result API by returning an empty payload if that invariant changes.
    if encoder.write_all(data.as_bytes()).is_err() {
        return Vec::new();
    }
    let Ok(bytes) = encoder.finish() else {
        return Vec::new();
    };
    bytes
}

pub fn gzip_decompress_bytes(data: &[u8]) -> Result<String, std::io::Error> {
    let mut decoder = GzDecoder::new(data);
    let mut out = String::new();
    decoder.read_to_string(&mut out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{gzip_compress_bytes, gzip_decompress_bytes};

    #[test]
    fn gzip_adapter_round_trips_text() {
        let compressed = gzip_compress_bytes("sifr gzip adapter");
        assert!(!compressed.is_empty());
        let decompressed = gzip_decompress_bytes(&compressed).expect("gzip should decompress");
        assert_eq!(decompressed, "sifr gzip adapter");
    }

    #[test]
    fn gzip_adapter_reports_invalid_data() {
        let err = gzip_decompress_bytes(&[]).expect_err("empty gzip payload should fail");
        assert!(!err.to_string().is_empty());
    }
}
