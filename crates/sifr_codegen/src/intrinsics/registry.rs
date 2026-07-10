mod bytes;
mod collections;
mod encoding;
mod file_handles;
mod open_text_handles;
mod requirements;
mod runtime;
mod task;
mod test;

use crate::RustExpr;
use sifr_stdlib_manifest::StdlibFeature;

pub(crate) use requirements::additional_required_features;

pub(crate) struct LoweredIntrinsic {
    pub(crate) expr: RustExpr,
    pub(crate) required_feature: Option<StdlibFeature>,
    pub(crate) additional_required_features: &'static [StdlibFeature],
}

pub(crate) fn lower_intrinsic(name: &str, args: &[RustExpr]) -> Option<LoweredIntrinsic> {
    lower_intrinsic_rendered(name, args)
}

pub(crate) fn lower_intrinsic_rendered(name: &str, args: &[RustExpr]) -> Option<LoweredIntrinsic> {
    let (expr, required_feature) = match name {
        "builtin_open" => (file_handles::lower_builtin_open(args), None),
        "builtin_open_text" => (open_text_handles::lower_builtin_open_text(args), None),
        "assert_eq" => (test::lower_assert_eq(args), None),
        "assert_ne" => (test::lower_assert_ne(args), None),
        "assert_true" => (test::lower_assert_true(args), None),
        "assert_false" => (test::lower_assert_false(args), None),
        "assert_almost_eq" => (test::lower_assert_almost_eq(args), None),
        "assert_gt" => (test::lower_assert_gt(args), None),
        "assert_lt" => (test::lower_assert_lt(args), None),
        "counter_from_list" => (
            collections::lower_counter_from_list(args),
            Some(StdlibFeature::SerdeJson),
        ),
        "counter_get" => (
            collections::lower_counter_get(args),
            Some(StdlibFeature::SerdeJson),
        ),
        "counter_most_common" => (
            collections::lower_counter_most_common(args),
            Some(StdlibFeature::SerdeJson),
        ),
        "counter_total" => (
            collections::lower_counter_total(args),
            Some(StdlibFeature::SerdeJson),
        ),
        "counter_values" => (
            collections::lower_counter_values(args),
            Some(StdlibFeature::SerdeJson),
        ),
        "counter_keys" => (
            collections::lower_counter_keys(args),
            Some(StdlibFeature::SerdeJson),
        ),
        "counter_items" => (
            collections::lower_counter_items(args),
            Some(StdlibFeature::SerdeJson),
        ),
        "counter_increment" => (
            collections::lower_counter_increment(args),
            Some(StdlibFeature::SerdeJson),
        ),
        "str_encode_utf8_result" => (
            encoding::lower_str_encode_result(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "str_encode_utf8_result_with_encoding" => (
            encoding::lower_str_encode_result(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "decode_utf8" => (
            encoding::lower_bytes_decode_result(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "decode_utf8_with_encoding" => (
            encoding::lower_bytes_decode_result(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "bytes_to_hex_strict" => (bytes::lower_bytes_to_hex_strict(args), None),
        "bytes_from_hex" => (bytes::lower_bytes_from_hex(args), None),
        "bytes_with_size" => (bytes::lower_bytes_with_size(args), None),
        "bytes_from_ints" => (bytes::lower_bytes_from_ints(args), None),
        "runtime_emit_diagnostic" => (runtime::lower_runtime_emit_diagnostic(args), None),
        "task_current_context" => (
            task::lower_task_current_context(args),
            Some(StdlibFeature::Tokio),
        ),
        _ => return None,
    };

    Some(LoweredIntrinsic {
        expr: expr?,
        required_feature,
        additional_required_features: additional_required_features(name),
    })
}
