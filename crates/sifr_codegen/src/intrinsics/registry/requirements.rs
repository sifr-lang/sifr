use sifr_stdlib_model::StdlibFeature;

use super::tls;

const HTTP_PRIMITIVE_REQUIRED_FEATURES: &[StdlibFeature] = &[StdlibFeature::Http];

const HTTP_HEADER_REQUIRED_FEATURES: &[StdlibFeature] = &[StdlibFeature::Http];

const HTTP_TRANSPORT_REQUIRED_FEATURES: &[StdlibFeature] = &[
    StdlibFeature::SifrRuntime,
    StdlibFeature::Tokio,
    StdlibFeature::TokioRustls,
    StdlibFeature::Rustls,
    StdlibFeature::RustlsPemfile,
    StdlibFeature::RustlsPlatformVerifier,
    StdlibFeature::Tracing,
    StdlibFeature::Bytes,
    StdlibFeature::Http,
    StdlibFeature::HttpBody,
    StdlibFeature::HttpBodyUtil,
    StdlibFeature::Hyper,
    StdlibFeature::HyperUtil,
    StdlibFeature::H2,
    StdlibFeature::TowerService,
];

pub(crate) fn additional_required_features(name: &str) -> &'static [StdlibFeature] {
    match name {
        // random_gauss uses rand_distr::Normal in addition to rand::rng.
        "random_gauss" => &[StdlibFeature::RandDistr],
        "json_loads"
        | "json_validate_integer_digit_limits"
        | "json_dumps_value_exact"
        | "json_dumps_value_web"
        | "json_dumps_value_string_ints" => &[StdlibFeature::SifrRuntime],
        "encoding_is_supported"
        | "encoding_canonical_label"
        | "encoding_decode_text"
        | "encoding_decode_recoveries"
        | "encoding_decode_outcome"
        | "encoding_decode_incremental_outcome"
        | "encoding_decode_incremental_pending"
        | "encoding_encode_bytes"
        | "encoding_encode_recoveries"
        | "encoding_encode_outcome"
        | "str_encode_utf8_result"
        | "str_encode_utf8_result_with_encoding"
        | "decode_utf8"
        | "decode_utf8_with_encoding"
        | "process_output_text"
        | "process_shell_output_text" => &[StdlibFeature::EncodingRs],
        "runtime_emit_diagnostic" => &[StdlibFeature::Metrics, StdlibFeature::Tracing],
        "http_validate_method" | "http_validate_status" | "http_validate_version" => {
            HTTP_PRIMITIVE_REQUIRED_FEATURES
        }
        "http_validate_header_name"
        | "http_validate_header_value"
        | "http_header_map_from_pairs" => HTTP_PRIMITIVE_REQUIRED_FEATURES,
        "http_parse_cookie_header" | "http_build_cookie_header" => HTTP_HEADER_REQUIRED_FEATURES,
        "http1_client_roundtrip_tcp"
        | "http1_server_respond_tcp"
        | "http2_client_roundtrip_tcp"
        | "http2_server_respond_tcp"
        | "http1_client_roundtrip_tls"
        | "http1_server_respond_tls"
        | "http2_client_roundtrip_tls"
        | "http2_server_respond_tls" => HTTP_TRANSPORT_REQUIRED_FEATURES,
        name if name.starts_with("net_") => &[StdlibFeature::SifrRuntime],
        name if name.starts_with("tls_") => tls::TLS_REQUIRED_FEATURES,
        name if name.starts_with("http_") => HTTP_HEADER_REQUIRED_FEATURES,
        "unicode_data_version"
        | "unicode_normalize"
        | "unicode_is_normalized"
        | "unicode_name"
        | "unicode_lookup"
        | "unicode_category"
        | "unicode_bidirectional"
        | "unicode_combining"
        | "unicode_east_asian_width"
        | "unicode_mirrored"
        | "unicode_decomposition"
        | "unicode_decimal"
        | "unicode_digit"
        | "unicode_numeric_value"
        | "unicode_case_fold"
        | "unicode_graphemes"
        | "unicode_grapheme_indices"
        | "unicode_words"
        | "unicode_word_boundaries" => &[
            StdlibFeature::UnicodeNames,
            StdlibFeature::UnicodeNormalization,
            StdlibFeature::UnicodeSegmentation,
        ],
        "i18n_locale_canonicalize"
        | "i18n_locale_maximize"
        | "i18n_locale_minimize"
        | "i18n_host_locale"
        | "i18n_format_number"
        | "i18n_format_datetime"
        | "i18n_plural_category"
        | "i18n_collate"
        | "i18n_mo_validate"
        | "i18n_mo_load_file"
        | "i18n_mo_lookup"
        | "i18n_mo_lookup_context"
        | "i18n_mo_lookup_plural"
        | "i18n_mo_lookup_context_plural" => &[
            StdlibFeature::IcuCollator,
            StdlibFeature::IcuDatetime,
            StdlibFeature::IcuDecimal,
            StdlibFeature::IcuLocale,
            StdlibFeature::IcuPlurals,
        ],
        _ => &[],
    }
}
