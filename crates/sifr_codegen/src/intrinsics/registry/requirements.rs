use sifr_stdlib_manifest::StdlibFeature;

pub(crate) fn additional_required_features(name: &str) -> &'static [StdlibFeature] {
    match name {
        "str_encode_utf8_result"
        | "str_encode_utf8_result_with_encoding"
        | "decode_utf8"
        | "decode_utf8_with_encoding" => &[StdlibFeature::EncodingRs],
        "runtime_emit_diagnostic" => &[StdlibFeature::Metrics, StdlibFeature::Tracing],
        _ => &[],
    }
}
