//! Text encoding adapters for generated Sifr programs.

#[must_use]
pub const fn feature_name() -> &'static str {
    "encoding"
}

#[must_use]
pub fn encoding_is_supported(label: &str) -> bool {
    sifr_runtime::encoding::is_supported_encoding(label)
}

pub fn encoding_canonical_label(label: &str) -> Result<String, String> {
    sifr_runtime::encoding::canonical_label(label)
}

pub fn encoding_decode_text(data: &[u8], encoding: &str, errors: &str) -> Result<String, String> {
    sifr_runtime::encoding::decode_text(data, encoding, errors)
}

pub fn encoding_decode_recoveries(
    data: &[u8],
    encoding: &str,
    errors: &str,
) -> Result<Vec<String>, String> {
    sifr_runtime::encoding::decode_recoveries(data, encoding, errors)
}

pub fn encoding_decode_incremental_text(
    data: &[u8],
    pending: &[u8],
    encoding: &str,
    errors: &str,
    final_chunk: bool,
) -> Result<String, String> {
    let (text, _) = sifr_runtime::encoding::incremental_decode_with_recoveries(
        data,
        pending,
        encoding,
        errors,
        final_chunk,
    )?;
    Ok(text)
}

pub fn encoding_decode_incremental_recoveries(
    data: &[u8],
    pending: &[u8],
    encoding: &str,
    errors: &str,
    final_chunk: bool,
) -> Result<Vec<String>, String> {
    let (_, recoveries) = sifr_runtime::encoding::incremental_decode_with_recoveries(
        data,
        pending,
        encoding,
        errors,
        final_chunk,
    )?;
    Ok(recoveries)
}

pub fn encoding_decode_incremental_pending(
    data: &[u8],
    pending: &[u8],
    encoding: &str,
    final_chunk: bool,
) -> Result<Vec<u8>, String> {
    sifr_runtime::encoding::incremental_decode_pending(data, pending, encoding, final_chunk)
}

pub fn encoding_encode_bytes(text: &str, encoding: &str, errors: &str) -> Result<Vec<u8>, String> {
    sifr_runtime::encoding::encode_bytes(text, encoding, errors)
}

pub fn encoding_encode_recoveries(
    text: &str,
    encoding: &str,
    errors: &str,
) -> Result<Vec<String>, String> {
    sifr_runtime::encoding::encode_recoveries(text, encoding, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoding_adapter_delegates_text_codec_behavior() {
        assert_eq!(feature_name(), "encoding");
        assert!(encoding_is_supported("utf_8"));
        assert_eq!(encoding_canonical_label("cp1252").unwrap(), "windows-1252");

        let encoded = encoding_encode_bytes("cafe", "ascii", "strict").unwrap();
        assert_eq!(encoded, b"cafe");
        assert_eq!(
            encoding_decode_text(&encoded, "ascii", "strict").unwrap(),
            "cafe"
        );

        let replaced = encoding_encode_bytes("cafe\u{301}", "ascii", "replace").unwrap();
        let recoveries = encoding_encode_recoveries("cafe\u{301}", "ascii", "replace").unwrap();
        assert_eq!(replaced, b"cafe?");
        assert_eq!(recoveries.len(), 1);
    }

    #[test]
    fn incremental_decode_adapter_preserves_pending_tail() {
        let first =
            encoding_decode_incremental_text(&[0xE2], &[], "utf-8", "strict", false).unwrap();
        let pending = encoding_decode_incremental_pending(&[0xE2], &[], "utf-8", false).unwrap();
        let second =
            encoding_decode_incremental_text(&[0x82, 0xAC], &pending, "utf-8", "strict", true)
                .unwrap();

        assert!(first.is_empty());
        assert_eq!(pending, vec![0xE2]);
        assert_eq!(second, "€");
    }
}
