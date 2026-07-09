#!/usr/bin/env python3
"""Guard stdlib migration closure invariants."""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
REGISTRY_DISPATCH_PATH = (
    REPO_ROOT / "crates" / "sifr_codegen" / "src" / "intrinsics" / "registry.rs"
)
DELETED_OWNERSHIP_REGISTRY = REPO_ROOT / "internal_docs" / "stdlib_native_surface_ownership.toml"
ARCH_DOC_PATH = REPO_ROOT / "internal_docs" / "sifr_sysroot_and_stdlib_architecture.md"

# This guard intentionally scans only registry.rs match-arm patterns, where
# string literals are the active intrinsic dispatch surface. It should not be
# used as a general Rust parser.
DISPATCH_PATTERN_RE = re.compile(
    r'(?:"([A-Za-z0-9_]+)"|Named\("([A-Za-z0-9_]+)"\))'
)

RETIRED_INTRINSICS = frozenset(
    {
        "atan2",
        "append_text",
        "b32decode",
        "b32encode",
        "b32hexdecode",
        "b32hexencode",
        "base64_decode",
        "base64_decode_bytes",
        "base64_decode_opts",
        "base64_encode",
        "base64_encode_bytes",
        "base64_encode_opts",
        "blake2b",
        "blake2b_bytes",
        "blake2s",
        "blake2s_bytes",
        "bytes_to_hex",
        "calendar_isleap",
        "calendar_monthrange",
        "calendar_weekday",
        "ceil",
        "cpu_count",
        "copy_file",
        "datetime_format",
        "datetime_from_timestamp",
        "datetime_now",
        "datetime_now_struct",
        "defaultdict_get",
        "defaultdict_new",
        "defaultdict_set",
        "dist",
        "env_get",
        "env_items",
        "env_keys",
        "env_set",
        "env_unset",
        "env_values",
        "encode_utf8",
        "encoding_canonical_label",
        "encoding_decode_incremental_outcome",
        "encoding_decode_incremental_pending",
        "encoding_decode_outcome",
        "encoding_decode_recoveries",
        "encoding_decode_text",
        "encoding_encode_bytes",
        "encoding_encode_outcome",
        "encoding_encode_recoveries",
        "encoding_is_supported",
        "exists",
        "erf",
        "erfc",
        "file_close",
        "file_read",
        "file_read_bytes",
        "file_readline",
        "file_readlines",
        "file_write",
        "file_write_bytes",
        "floor",
        "frexp",
        "fsum",
        "gamma",
        "get_args",
        "get_global_level",
        "getpid",
        "getcwd",
        "gettempdir",
        "glob_pattern",
        "gzip_compress",
        "_gzip_compress_bytes_impl",
        "gzip_decompress",
        "_gzip_decompress_bytes_impl",
        "html_escape",
        "html_unescape",
        "http_build_cookie_header",
        "http_header_map_from_pairs",
        "http_parse_cookie_header",
        "http_validate_header_name",
        "http_validate_header_value",
        "http_validate_method",
        "http_validate_status",
        "http_validate_version",
        "i18n_collate",
        "i18n_format_datetime",
        "i18n_format_number",
        "i18n_host_locale",
        "i18n_locale_canonicalize",
        "i18n_locale_maximize",
        "i18n_locale_minimize",
        "i18n_mo_load_file",
        "i18n_mo_lookup",
        "i18n_mo_lookup_context",
        "i18n_mo_lookup_context_plural",
        "i18n_mo_lookup_plural",
        "i18n_mo_validate",
        "i18n_plural_category",
        "is_dir",
        "is_file",
        "isfinite",
        "isqrt",
        "iterdir",
        "json_dump_tokens",
        "json_dump_tokens_exact",
        "json_dump_tokens_string_ints",
        "json_dump_tokens_web",
        "json_dumps",
        "json_dumps_value",
        "json_dumps_value_exact",
        "json_dumps_value_string_ints",
        "json_dumps_value_web",
        "json_load_tokens",
        "json_loads",
        "json_validate_integer_digit_limits",
        "ldexp",
        "lgamma",
        "listdir",
        "makedirs",
        "md5",
        "md5_bytes",
        "mkdir",
        "modf",
        "monotonic",
        "net_connect_tcp",
        "net_listen_tcp",
        "net_listener_accept",
        "net_listener_close",
        "net_listener_local_addr",
        "net_lookup_host",
        "net_tcp_read_half_close",
        "net_tcp_read_half_read_chunk",
        "net_tcp_stream_close",
        "net_tcp_stream_local_addr",
        "net_tcp_stream_peer_addr",
        "net_tcp_stream_read_chunk",
        "net_tcp_stream_shutdown_write",
        "net_tcp_stream_split",
        "net_tcp_stream_write",
        "net_tcp_stream_write_all",
        "net_tcp_write_half_close",
        "net_tcp_write_half_shutdown_write",
        "net_tcp_write_half_write",
        "net_tcp_write_half_write_all",
        "nextafter",
        "open_file",
        "os_linesep",
        "os_name",
        "os_sep",
        "platform_arch",
        "platform_node",
        "platform_processor",
        "platform_release",
        "platform_system",
        "platform_version",
        "pow_val",
        "py_from_bool",
        "py_from_bytes",
        "py_from_float",
        "py_from_int",
        "py_from_none",
        "py_from_str",
        "py_to_bool",
        "py_to_bytes",
        "py_to_float",
        "py_to_i16",
        "py_to_i32",
        "py_to_i64",
        "py_to_i8",
        "py_to_int",
        "py_to_isize",
        "py_to_none",
        "py_to_str",
        "py_to_u16",
        "py_to_u32",
        "py_to_u64",
        "py_to_u8",
        "py_to_usize",
        "random_choice",
        "random_float",
        "random_gauss",
        "random_int",
        "random_module_set_state",
        "random_module_state_gauss_next",
        "random_module_state_index",
        "random_module_state_words",
        "random_randrange",
        "random_sample",
        "random_shuffle",
        "random_uniform",
        "read_lines",
        "read_text",
        "remove_file",
        "rename",
        "regex_match",
        "regex_search",
        "regex_split",
        "regex_sub",
        "remainder",
        "resolve_path",
        "rglob_pattern",
        "rmdir",
        "rmdir_all",
        "round_val",
        "set_add",
        "set_contains",
        "set_from_list",
        "set_global_level",
        "set_intersection",
        "set_len",
        "set_remove",
        "set_union",
        "sha1",
        "sha1_bytes",
        "signal_ctrl_c",
        "signal_shutdown",
        "signal_terminate",
        "sha224",
        "sha224_bytes",
        "sha256",
        "sha256_bytes",
        "sha384",
        "sha384_bytes",
        "sha512",
        "sha512_bytes",
        "sleep",
        "sqrt",
        "sys_exit",
        "sys_maxsize",
        "sys_platform",
        "sys_version",
        "tls_accept",
        "tls_client_config_close",
        "tls_client_config_platform",
        "tls_client_config_with_roots",
        "tls_client_config_with_roots_and_client_auth",
        "tls_connect",
        "tls_read_half_close",
        "tls_read_half_read_chunk",
        "tls_server_config",
        "tls_server_config_close",
        "tls_server_config_require_client_auth",
        "tls_stream_alpn_protocol",
        "tls_stream_close",
        "tls_stream_close_notify",
        "tls_stream_flush",
        "tls_stream_protocol_version",
        "tls_stream_read_chunk",
        "tls_stream_split",
        "tls_stream_write",
        "tls_stream_write_all",
        "tls_write_half_close",
        "tls_write_half_close_notify",
        "tls_write_half_flush",
        "tls_write_half_write",
        "tls_write_half_write_all",
        "_gmtime_intrinsic",
        "_localtime_intrinsic",
        "_strptime_intrinsic",
        "gmtime",
        "localtime",
        "sumprod",
        "perf_counter",
        "strptime",
        "time_format",
        "time_gmtime",
        "time_localtime",
        "time_now",
        "time_strptime",
        "touch",
        "toml_parse",
        "toml_parse_tokens",
        "ulp",
        "unicode_bidirectional",
        "unicode_case_fold",
        "unicode_category",
        "unicode_combining",
        "unicode_data_version",
        "unicode_decimal",
        "unicode_decomposition",
        "unicode_digit",
        "unicode_east_asian_width",
        "unicode_grapheme_indices",
        "unicode_graphemes",
        "unicode_is_normalized",
        "unicode_lookup",
        "unicode_mirrored",
        "unicode_name",
        "unicode_normalize",
        "unicode_numeric_value",
        "unicode_word_boundaries",
        "unicode_words",
        "url_build_parts",
        "url_normalize_path",
        "url_parse_parts",
        "url_percent_decode",
        "url_percent_decode_bytes",
        "url_percent_encode",
        "url_percent_encode_bytes",
        "url_query_build_flat",
        "url_query_parse_flat",
        "urlsafe_b64decode",
        "urlsafe_b64decode_bytes",
        "urlsafe_b64encode",
        "urlsafe_b64encode_bytes",
        "uuid3_text",
        "uuid4",
        "uuid5_text",
        "walk_dir",
        "which",
        "write_text",
        "zip_add_file",
        "zip_add_file_bytes",
        "zip_create",
        "zip_namelist",
        "zip_read_file",
        "zip_read_file_bytes",
    }
)

STALE_ARCH_PHRASES = (
    "complete surface-by-surface ownership decision remains the TOML registry",
    "validated against the compiler intrinsic registry; compiler intrinsic metadata remains the current signature owner",
    "old handwritten intrinsic registry is removed or reduced",
)


def main() -> int:
    failures = _validate(
        _read_text(REGISTRY_DISPATCH_PATH),
        DELETED_OWNERSHIP_REGISTRY.exists(),
        _read_text(ARCH_DOC_PATH),
    )
    if failures:
        print("stdlib migration closure guard: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1

    print(
        "stdlib migration closure guard: PASS "
        f"(retired_intrinsics={len(RETIRED_INTRINSICS)}, "
        f"registry_file_deleted={not DELETED_OWNERSHIP_REGISTRY.exists()})"
    )
    return 0


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _validate(registry_text: str, deleted_registry_exists: bool, arch_doc_text: str) -> list[str]:
    failures: list[str] = []

    active_intrinsics = _active_intrinsic_names(registry_text)
    reintroduced = sorted(active_intrinsics & RETIRED_INTRINSICS)
    if reintroduced:
        failures.append(
            "retired migrated intrinsics are active in registry.rs: "
            + ", ".join(reintroduced)
        )

    if deleted_registry_exists:
        failures.append(
            "internal_docs/stdlib_native_surface_ownership.toml must remain deleted"
        )

    for phrase in STALE_ARCH_PHRASES:
        if phrase in arch_doc_text:
            failures.append(f"architecture doc contains stale deleted-registry phrase: {phrase!r}")

    return failures


def _active_intrinsic_names(registry_text: str) -> set[str]:
    active: set[str] = set()
    pending_pattern = ""
    for line in registry_text.splitlines():
        if "=>" not in line:
            if '"' in line or "Named(" in line:
                pending_pattern = f"{pending_pattern} {line.strip()}".strip()
            continue

        before_arrow = line.split("=>", 1)[0]
        pattern = f"{pending_pattern} {before_arrow}".strip()
        pending_pattern = ""
        pattern = re.split(r"\s+if\b", pattern, maxsplit=1)[0]
        active.update(first or second for first, second in DISPATCH_PATTERN_RE.findall(pattern))

    return active



def _self_test() -> int:
    if _validate('"sqrt" => lower_sqrt(args),', False, "clean architecture text") != [
        "retired migrated intrinsics are active in registry.rs: sqrt"
    ]:
        print("self-test active retired intrinsic was not rejected", file=sys.stderr)
        return 1

    if _validate('"encode_utf8" if cfg!(test) => lower_encode_utf8(args),', False, "") != [
        "retired migrated intrinsics are active in registry.rs: encode_utf8"
    ]:
        print("self-test guarded retired intrinsic was not rejected", file=sys.stderr)
        return 1

    equality_guard = '"sqrt" if kind == Kind::Real => lower_sqrt(args),'
    if _validate(equality_guard, False, "") != [
        "retired migrated intrinsics are active in registry.rs: sqrt"
    ]:
        print("self-test equality-guard retired intrinsic was not rejected", file=sys.stderr)
        return 1

    if _validate('IntrinsicKind::Named("set_add") => lower_set_add(args),', False, "") != [
        "retired migrated intrinsics are active in registry.rs: set_add"
    ]:
        print("self-test named retired intrinsic was not rejected", file=sys.stderr)
        return 1

    non_pattern = '_ if name == "sqrt" || other() => lower_fallback(args),'
    if _validate(non_pattern, False, ""):
        print("self-test guard-only retired string should pass", file=sys.stderr)
        return 1

    stale_phrase = STALE_ARCH_PHRASES[0]
    if not any(
        "stale deleted-registry phrase" in failure
        for failure in _validate("", False, stale_phrase)
    ):
        print("self-test stale architecture phrase was not rejected", file=sys.stderr)
        return 1

    if _validate("", True, "") != [
        "internal_docs/stdlib_native_surface_ownership.toml must remain deleted"
    ]:
        print("self-test restored ownership registry was not rejected", file=sys.stderr)
        return 1

    if _validate('"run_command" => lower_run_command(args),', False, "clean architecture text"):
        print("self-test retained intrinsic seed should pass", file=sys.stderr)
        return 1

    print("stdlib migration closure guard self-test: PASS")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        raise SystemExit(_self_test())
    raise SystemExit(main())
