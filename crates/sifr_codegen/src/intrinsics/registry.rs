mod bytes;
mod collections;
mod encoding;
mod file_handles;
mod net;
mod open_text_handles;
mod os;
mod python;
mod requirements;
mod runtime;
mod signal;
mod task;
mod test;
mod tls;
mod url_http;

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
        "run_command" => (os::lower_run_command(args), None),
        "chdir" => (os::lower_chdir(args), None),
        "stat_size" => (os::lower_stat_size(args), None),
        "disk_usage" => (os::lower_disk_usage(args), None),
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
        "http_validate_header_name" => (
            url_http::lower_http_validate_header_name(args),
            Some(StdlibFeature::Http),
        ),
        "http_validate_header_value" => (
            url_http::lower_http_validate_header_value(args),
            Some(StdlibFeature::Http),
        ),
        "http_header_map_from_pairs" => (
            url_http::lower_http_header_map_from_pairs(args),
            Some(StdlibFeature::Http),
        ),
        "http_parse_cookie_header" => (
            url_http::lower_http_parse_cookie_header(args),
            Some(StdlibFeature::Http),
        ),
        "http_build_cookie_header" => (
            url_http::lower_http_build_cookie_header(args),
            Some(StdlibFeature::Http),
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
        "net_connect_tcp" => (net::lower_net_connect_tcp(args), Some(StdlibFeature::Tokio)),
        "net_listen_tcp" => (net::lower_net_listen_tcp(args), Some(StdlibFeature::Tokio)),
        "net_lookup_host" => (net::lower_net_lookup_host(args), Some(StdlibFeature::Tokio)),
        "net_listener_accept" => (
            net::lower_net_listener_accept(args),
            Some(StdlibFeature::Tokio),
        ),
        "net_listener_local_addr" => (
            net::lower_net_listener_local_addr(args),
            Some(StdlibFeature::Tokio),
        ),
        "net_listener_close" => (
            net::lower_net_listener_close(args),
            Some(StdlibFeature::Tokio),
        ),
        "net_tcp_stream_read_chunk" => (
            net::lower_net_tcp_stream_read_chunk(args),
            Some(StdlibFeature::Tokio),
        ),
        "net_tcp_stream_write" => (
            net::lower_net_tcp_stream_write(args),
            Some(StdlibFeature::Tokio),
        ),
        "net_tcp_stream_write_all" => (
            net::lower_net_tcp_stream_write_all(args),
            Some(StdlibFeature::Tokio),
        ),
        "net_tcp_stream_shutdown_write" => (
            net::lower_net_tcp_stream_shutdown_write(args),
            Some(StdlibFeature::Tokio),
        ),
        "net_tcp_stream_split" => (
            net::lower_net_tcp_stream_split(args),
            Some(StdlibFeature::Tokio),
        ),
        "net_tcp_stream_local_addr" => (
            net::lower_net_tcp_stream_local_addr(args),
            Some(StdlibFeature::Tokio),
        ),
        "net_tcp_stream_peer_addr" => (
            net::lower_net_tcp_stream_peer_addr(args),
            Some(StdlibFeature::Tokio),
        ),
        "net_tcp_stream_close" => (
            net::lower_net_tcp_stream_close(args),
            Some(StdlibFeature::Tokio),
        ),
        "net_tcp_read_half_read_chunk" => (
            net::lower_net_tcp_read_half_read_chunk(args),
            Some(StdlibFeature::Tokio),
        ),
        "net_tcp_read_half_close" => (
            net::lower_net_tcp_read_half_close(args),
            Some(StdlibFeature::Tokio),
        ),
        "net_tcp_write_half_write" => (
            net::lower_net_tcp_write_half_write(args),
            Some(StdlibFeature::Tokio),
        ),
        "net_tcp_write_half_write_all" => (
            net::lower_net_tcp_write_half_write_all(args),
            Some(StdlibFeature::Tokio),
        ),
        "net_tcp_write_half_shutdown_write" => (
            net::lower_net_tcp_write_half_shutdown_write(args),
            Some(StdlibFeature::Tokio),
        ),
        "net_tcp_write_half_close" => (
            net::lower_net_tcp_write_half_close(args),
            Some(StdlibFeature::Tokio),
        ),
        name if name.starts_with("tls_") => (
            tls::lower_tls_intrinsic(name, args),
            Some(StdlibFeature::Tokio),
        ),
        name if name.starts_with("http_") => (
            url_http::lower_http_intrinsic(name, args),
            Some(StdlibFeature::Http),
        ),
        "signal_ctrl_c" => (
            signal::lower_signal_ctrl_c(args),
            Some(StdlibFeature::Tokio),
        ),
        "signal_terminate" => (
            signal::lower_signal_terminate(args),
            Some(StdlibFeature::Tokio),
        ),
        "signal_shutdown" => (
            signal::lower_signal_shutdown(args),
            Some(StdlibFeature::Tokio),
        ),
        "runtime_emit_diagnostic" => (runtime::lower_runtime_emit_diagnostic(args), None),
        name if name.starts_with("py_") => (
            python::lower_python_intrinsic(name, args),
            Some(StdlibFeature::PythonRuntime),
        ),
        "local_callback" | "threadsafe_callback" => (
            python::lower_python_intrinsic(name, args),
            Some(StdlibFeature::PythonRuntime),
        ),
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
