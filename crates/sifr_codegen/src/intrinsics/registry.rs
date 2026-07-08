mod bytes;
mod collections;
mod encoding;
mod env;
mod file_handles;
mod logging;
mod net;
mod open_text_handles;
mod os;
mod process;
mod process_async;
mod process_child_lifecycle;
mod process_pipes;
mod python;
mod random;
mod requirements;
mod runtime;
mod signal;
mod sys;
mod task;
mod test;
mod time;
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
        "env_get" => (env::lower_env_get(args), None),
        "env_set" => (env::lower_env_set(args), None),
        "env_unset" => (env::lower_env_unset(args), None),
        "env_keys" => (env::lower_env_keys(args), None),
        "env_values" => (env::lower_env_values(args), None),
        "env_items" => (env::lower_env_items(args), None),
        "run_command" => (os::lower_run_command(args), None),
        "get_args" => (os::lower_get_args(args), None),
        "chdir" => (os::lower_chdir(args), None),
        "getpid" => (os::lower_getpid(args), None),
        "cpu_count" => (os::lower_cpu_count(args), None),
        "stat_size" => (os::lower_stat_size(args), None),
        "which" => (os::lower_which(args), None),
        "disk_usage" => (os::lower_disk_usage(args), None),
        "os_sep" => (os::lower_os_sep(args), None),
        "os_linesep" => (os::lower_os_linesep(args), None),
        "os_name" => (os::lower_os_name(args), None),
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
        "time_now" => (time::lower_time_now(args), None),
        "sleep" => (time::lower_sleep(args), None),
        "time_format" => (time::lower_time_format(args), Some(StdlibFeature::Chrono)),
        "perf_counter" => (time::lower_perf_counter(args), None),
        "monotonic" => (time::lower_monotonic(args), None),
        "strptime" => (time::lower_strptime(args), Some(StdlibFeature::Chrono)),
        "gmtime" => (time::lower_gmtime(args), Some(StdlibFeature::Chrono)),
        "localtime" => (time::lower_localtime(args), Some(StdlibFeature::Chrono)),
        "_strptime_intrinsic" => (time::lower_strptime(args), Some(StdlibFeature::Chrono)),
        "_gmtime_intrinsic" => (time::lower_gmtime(args), Some(StdlibFeature::Chrono)),
        "_localtime_intrinsic" => (time::lower_localtime(args), Some(StdlibFeature::Chrono)),
        "time_strptime" => (
            time::lower_time_strptime_parts(args),
            Some(StdlibFeature::Chrono),
        ),
        "time_gmtime" => (
            time::lower_time_gmtime_parts(args),
            Some(StdlibFeature::Chrono),
        ),
        "time_localtime" => (
            time::lower_time_localtime_parts(args),
            Some(StdlibFeature::Chrono),
        ),
        "random_int" => (random::lower_random_int(args), Some(StdlibFeature::Rand)),
        "random_float" => (random::lower_random_float(args), Some(StdlibFeature::Rand)),
        "random_choice" => (random::lower_random_choice(args), Some(StdlibFeature::Rand)),
        "random_uniform" => (
            random::lower_random_uniform(args),
            Some(StdlibFeature::Rand),
        ),
        "random_shuffle" => (
            random::lower_random_shuffle(args),
            Some(StdlibFeature::Rand),
        ),
        "random_sample" => (random::lower_random_sample(args), Some(StdlibFeature::Rand)),
        "random_randrange" => (
            random::lower_random_randrange(args),
            Some(StdlibFeature::Rand),
        ),
        "random_gauss" => (random::lower_random_gauss(args), Some(StdlibFeature::Rand)),
        "random_module_state_words" => (random::lower_random_module_state_words(args), None),
        "random_module_state_index" => (random::lower_random_module_state_index(args), None),
        "random_module_state_gauss_next" => {
            (random::lower_random_module_state_gauss_next(args), None)
        }
        "random_module_set_state" => (random::lower_random_module_set_state(args), None),
        "sys_exit" => (sys::lower_sys_exit(args), None),
        "sys_version" => (sys::lower_sys_version(args), None),
        "sys_platform" => (sys::lower_sys_platform(args), None),
        "sys_maxsize" => (sys::lower_sys_maxsize(args), None),
        "process_run" => (process::lower_process_run(args), None),
        "process_spawn" => (process_child_lifecycle::lower_process_spawn(args), None),
        "process_kill" => (process_child_lifecycle::lower_process_kill(args), None),
        "process_terminate" => (process_child_lifecycle::lower_process_terminate(args), None),
        "process_wait" => (process_child_lifecycle::lower_process_wait(args), None),
        "process_child_stdin" => (process_pipes::lower_process_child_stdin(args), None),
        "process_child_stdout" => (process_pipes::lower_process_child_stdout(args), None),
        "process_child_stderr" => (process_pipes::lower_process_child_stderr(args), None),
        "process_pipe_read_all" => (process_pipes::lower_process_pipe_read_all(args), None),
        "process_pipe_read" => (process_pipes::lower_process_pipe_read(args), None),
        "process_pipe_reader_close" => (process_pipes::lower_process_pipe_reader_close(args), None),
        "process_pipe_write_all" => (process_pipes::lower_process_pipe_write_all(args), None),
        "process_pipe_close" => (process_pipes::lower_process_pipe_close(args), None),
        "process_output" => (process::lower_process_output(args), None),
        "process_output_text" => (process::lower_process_output_text(args), None),
        "process_output_timeout" => (process::lower_process_output_timeout(args), None),
        "process_async_run" => (
            process_async::lower_process_async_run(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_async_run_timeout" => (
            process_async::lower_process_async_run_timeout(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_async_output" => (
            process_async::lower_process_async_output(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_async_output_timeout" => (
            process_async::lower_process_async_output_timeout(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_async_spawn" => (
            process_async::lower_process_async_spawn(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_async_wait" => (
            process_async::lower_process_async_wait(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_handle_wait" => (
            process_async::lower_process_handle_wait(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_async_kill" => (
            process_async::lower_process_async_kill(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_async_terminate" => (
            process_async::lower_process_async_terminate(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_async_child_stdin" => (
            process_async::lower_process_async_child_stdin(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_async_child_stdout" => (
            process_async::lower_process_async_child_stdout(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_async_child_stderr" => (
            process_async::lower_process_async_child_stderr(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_async_pipe_read_all" => (
            process_async::lower_process_async_pipe_read_all(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_async_pipe_read" => (
            process_async::lower_process_async_pipe_read(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_async_pipe_reader_close" => (
            process_async::lower_process_async_pipe_reader_close(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_async_pipe_write_all" => (
            process_async::lower_process_async_pipe_write_all(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_async_pipe_close" => (
            process_async::lower_process_async_pipe_close(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_async_shell_run" => (
            process_async::lower_process_async_shell_run(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_async_shell_output" => (
            process_async::lower_process_async_shell_output(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_async_shell_output_timeout" => (
            process_async::lower_process_async_shell_output_timeout(args),
            Some(StdlibFeature::Tokio),
        ),
        "process_shell_run" => (process::lower_process_shell_run(args), None),
        "process_shell_output" => (process::lower_process_shell_output(args), None),
        "process_shell_output_text" => (process::lower_process_shell_output_text(args), None),
        "process_shell_output_timeout" => (process::lower_process_shell_output_timeout(args), None),
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
        "set_global_level" => (logging::lower_set_global_level(args), None),
        "get_global_level" => (logging::lower_get_global_level(args), None),
        _ => return None,
    };

    Some(LoweredIntrinsic {
        expr: expr?,
        required_feature,
        additional_required_features: additional_required_features(name),
    })
}
