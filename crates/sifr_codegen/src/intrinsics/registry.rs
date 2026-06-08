mod base32;
mod base64;
mod bytes;
mod calendar;
mod collections;
mod datetime;
mod digest_format;
mod encoding;
mod env;
mod file_handles;
mod gzip;
mod hash;
mod hashlib;
mod html;
mod i18n;
mod io;
mod json;
mod logging;
mod math;
mod open_text_handles;
mod os;
mod pathlib;
mod platform;
mod process;
mod process_async;
mod process_pipes;
mod random;
mod re;
mod sys;
mod test;
mod time;
mod toml;
mod unicode;
mod uuid;
mod zipfile;

use crate::RustExpr;
use sifr_stdlib::StdlibFeature;

pub(crate) struct LoweredIntrinsic {
    pub(crate) expr: RustExpr,
    pub(crate) required_feature: Option<StdlibFeature>,
    pub(crate) additional_required_features: &'static [StdlibFeature],
}

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
        | "decode_utf8_with_encoding" => &[StdlibFeature::EncodingRs],
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

pub(crate) fn lower_intrinsic(name: &str, args: &[RustExpr]) -> Option<LoweredIntrinsic> {
    lower_intrinsic_rendered(name, args)
}

pub(crate) fn lower_intrinsic_rendered(name: &str, args: &[RustExpr]) -> Option<LoweredIntrinsic> {
    let (expr, required_feature) = match name {
        "sqrt" => (math::lower_sqrt(args), None),
        "floor" => (math::lower_floor(args), None),
        "ceil" => (math::lower_ceil(args), None),
        "abs_val" => (math::lower_abs_val(args), None),
        "log" => (math::lower_log(args), None),
        "cbrt" => (math::lower_cbrt(args), None),
        "exp2" => (math::lower_exp2(args), None),
        "sin" => (math::lower_sin(args), None),
        "cos" => (math::lower_cos(args), None),
        "tan" => (math::lower_tan(args), None),
        "pow_val" => (math::lower_pow_val(args), None),
        "min_val" => (math::lower_min_val(args), None),
        "max_val" => (math::lower_max_val(args), None),
        "round_val" => (math::lower_round_val(args), None),
        "asin" => (math::lower_asin(args), None),
        "acos" => (math::lower_acos(args), None),
        "atan" => (math::lower_atan(args), None),
        "atan2" => (math::lower_atan2(args), None),
        "sinh" => (math::lower_sinh(args), None),
        "cosh" => (math::lower_cosh(args), None),
        "tanh" => (math::lower_tanh(args), None),
        "log10" => (math::lower_log10(args), None),
        "log2" => (math::lower_log2(args), None),
        "degrees" => (math::lower_degrees(args), None),
        "radians" => (math::lower_radians(args), None),
        "isnan" => (math::lower_isnan(args), None),
        "isinf" => (math::lower_isinf(args), None),
        "trunc" => (math::lower_trunc(args), None),
        "copysign" => (math::lower_copysign(args), None),
        "signbit" => (math::lower_signbit(args), None),
        "fmod" => (math::lower_fmod(args), None),
        "hypot" => (math::lower_hypot(args), None),
        "fma" => (math::lower_fma(args), None),
        "fmax" => (math::lower_fmax(args), None),
        "fmin" => (math::lower_fmin(args), None),
        "exp" => (math::lower_exp(args), None),
        "expm1" => (math::lower_expm1(args), None),
        "log1p" => (math::lower_log1p(args), None),
        "fabs" => (math::lower_fabs(args), None),
        "isfinite" => (math::lower_isfinite(args), None),
        "isnormal" => (math::lower_isnormal(args), None),
        "issubnormal" => (math::lower_issubnormal(args), None),
        "acosh" => (math::lower_acosh(args), None),
        "asinh" => (math::lower_asinh(args), None),
        "atanh" => (math::lower_atanh(args), None),
        "isqrt" => (math::lower_isqrt(args), None),
        "remainder" => (math::lower_remainder(args), None),
        "dist" => (math::lower_dist(args), None),
        "fsum" => (math::lower_fsum(args), None),
        "sumprod" => (math::lower_sumprod(args), None),
        "erf" => (math::lower_erf(args), None),
        "erfc" => (math::lower_erfc(args), None),
        "gamma" => (math::lower_gamma(args), None),
        "lgamma" => (math::lower_lgamma(args), None),
        "frexp" => (math::lower_frexp(args), None),
        "ldexp" => (math::lower_ldexp(args), None),
        "modf" => (math::lower_modf(args), None),
        "nextafter" => (math::lower_nextafter(args), None),
        "ulp" => (math::lower_ulp(args), None),
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
        "touch" => (pathlib::lower_touch(args), None),
        "resolve_path" => (pathlib::lower_resolve_path(args), None),
        "iterdir" => (pathlib::lower_iterdir(args), None),
        "glob_pattern" => (
            pathlib::lower_glob_pattern(args),
            Some(StdlibFeature::Regex),
        ),
        "rglob_pattern" => (
            pathlib::lower_rglob_pattern(args),
            Some(StdlibFeature::Regex),
        ),
        "read_text" => (io::lower_read_text(args), None),
        "write_text" => (io::lower_write_text(args), None),
        "exists" => (io::lower_exists(args), None),
        "read_lines" => (io::lower_read_lines(args), None),
        "append_text" => (io::lower_append_text(args), None),
        "getcwd" => (io::lower_getcwd(args), None),
        "listdir" => (io::lower_listdir(args), None),
        "mkdir" => (io::lower_mkdir(args), None),
        "rmdir" => (io::lower_rmdir(args), None),
        "remove_file" => (io::lower_remove_file(args), None),
        "rename" => (io::lower_rename(args), None),
        "is_file" => (io::lower_is_file(args), None),
        "is_dir" => (io::lower_is_dir(args), None),
        "copy_file" => (io::lower_copy_file(args), None),
        "walk_dir" => (io::lower_walk_dir(args), None),
        "rmdir_all" => (io::lower_rmdir_all(args), None),
        "gettempdir" => (io::lower_gettempdir(args), None),
        "makedirs" => (io::lower_makedirs(args), None),
        "builtin_open" => (file_handles::lower_builtin_open(args), None),
        "builtin_open_text" => (open_text_handles::lower_builtin_open_text(args), None),
        "open_file" => (file_handles::lower_open_file(args), None),
        "file_read" => (file_handles::lower_file_read(args), None),
        "file_write" => (file_handles::lower_file_write(args), None),
        "file_readline" => (file_handles::lower_file_readline(args), None),
        "file_readlines" => (file_handles::lower_file_readlines(args), None),
        "file_close" => (file_handles::lower_file_close(args), None),
        "file_read_bytes" => (file_handles::lower_file_read_bytes(args), None),
        "file_write_bytes" => (file_handles::lower_file_write_bytes(args), None),
        "json_loads" => (json::lower_json_loads(args), Some(StdlibFeature::SerdeJson)),
        "json_validate_integer_digit_limits" => {
            (json::lower_json_validate_integer_digit_limits(args), None)
        }
        "json_dumps" => (json::lower_json_dumps(args), Some(StdlibFeature::SerdeJson)),
        "json_dumps_value" => (
            json::lower_json_dumps_value(args),
            Some(StdlibFeature::SerdeJson),
        ),
        "json_dumps_value_exact" => (
            json::lower_json_dumps_value_exact(args),
            Some(StdlibFeature::SerdeJson),
        ),
        "json_dumps_value_web" => (
            json::lower_json_dumps_value_web(args),
            Some(StdlibFeature::SerdeJson),
        ),
        "json_dumps_value_string_ints" => (
            json::lower_json_dumps_value_string_ints(args),
            Some(StdlibFeature::SerdeJson),
        ),
        "assert_eq" => (test::lower_assert_eq(args), None),
        "assert_ne" => (test::lower_assert_ne(args), None),
        "assert_true" => (test::lower_assert_true(args), None),
        "assert_false" => (test::lower_assert_false(args), None),
        "assert_almost_eq" => (test::lower_assert_almost_eq(args), None),
        "assert_gt" => (test::lower_assert_gt(args), None),
        "assert_lt" => (test::lower_assert_lt(args), None),
        "new_set" => (collections::lower_new_set(args), None),
        "set_from_list" => (collections::lower_set_from_list(args), None),
        "set_add" => (collections::lower_set_add(args), None),
        "set_contains" => (collections::lower_set_contains(args), None),
        "set_remove" => (collections::lower_set_remove(args), None),
        "set_len" => (collections::lower_set_len(args), None),
        "set_union" => (collections::lower_set_union(args), None),
        "set_intersection" => (collections::lower_set_intersection(args), None),
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
        "defaultdict_new" => (collections::lower_defaultdict_new(args), None),
        "defaultdict_get" => (
            collections::lower_defaultdict_get(args),
            Some(StdlibFeature::SerdeJson),
        ),
        "defaultdict_set" => (
            collections::lower_defaultdict_set(args),
            Some(StdlibFeature::SerdeJson),
        ),
        "encoding_is_supported" => (
            encoding::lower_encoding_is_supported(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "encoding_canonical_label" => (
            encoding::lower_encoding_canonical_label(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "encoding_decode_text" => (
            encoding::lower_encoding_decode_text(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "encoding_decode_recoveries" => (
            encoding::lower_encoding_decode_recoveries(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "encoding_decode_outcome" => (
            encoding::lower_encoding_decode_outcome(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "encoding_decode_incremental_outcome" => (
            encoding::lower_encoding_decode_incremental_outcome(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "encoding_decode_incremental_pending" => (
            encoding::lower_encoding_decode_incremental_pending(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "encoding_encode_bytes" => (
            encoding::lower_encoding_encode_bytes(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "encoding_encode_recoveries" => (
            encoding::lower_encoding_encode_recoveries(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "encoding_encode_outcome" => (
            encoding::lower_encoding_encode_outcome(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_data_version" => (
            unicode::lower_unicode_data_version(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_normalize" => (
            unicode::lower_unicode_normalize(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_is_normalized" => (
            unicode::lower_unicode_is_normalized(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_name" => (
            unicode::lower_unicode_name(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_lookup" => (
            unicode::lower_unicode_lookup(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_category" => (
            unicode::lower_unicode_category(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_bidirectional" => (
            unicode::lower_unicode_bidirectional(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_combining" => (
            unicode::lower_unicode_combining(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_east_asian_width" => (
            unicode::lower_unicode_east_asian_width(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_mirrored" => (
            unicode::lower_unicode_mirrored(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_decomposition" => (
            unicode::lower_unicode_decomposition(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_decimal" => (
            unicode::lower_unicode_decimal(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_digit" => (
            unicode::lower_unicode_digit(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_numeric_value" => (
            unicode::lower_unicode_numeric_value(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_case_fold" => (
            unicode::lower_unicode_case_fold(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_graphemes" => (
            unicode::lower_unicode_graphemes(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_grapheme_indices" => (
            unicode::lower_unicode_grapheme_indices(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_words" => (
            unicode::lower_unicode_words(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "unicode_word_boundaries" => (
            unicode::lower_unicode_word_boundaries(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "i18n_locale_canonicalize" => (
            i18n::lower_i18n_locale_canonicalize(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "i18n_locale_maximize" => (
            i18n::lower_i18n_locale_maximize(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "i18n_locale_minimize" => (
            i18n::lower_i18n_locale_minimize(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "i18n_host_locale" => (
            i18n::lower_i18n_host_locale(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "i18n_format_number" => (
            i18n::lower_i18n_format_number(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "i18n_format_datetime" => (
            i18n::lower_i18n_format_datetime(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "i18n_plural_category" => (
            i18n::lower_i18n_plural_category(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "i18n_collate" => (
            i18n::lower_i18n_collate(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "i18n_mo_validate" => (
            i18n::lower_i18n_mo_validate(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "i18n_mo_load_file" => (
            i18n::lower_i18n_mo_load_file(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "i18n_mo_lookup" => (
            i18n::lower_i18n_mo_lookup(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "i18n_mo_lookup_context" => (
            i18n::lower_i18n_mo_lookup_context(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "i18n_mo_lookup_plural" => (
            i18n::lower_i18n_mo_lookup_plural(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "i18n_mo_lookup_context_plural" => (
            i18n::lower_i18n_mo_lookup_context_plural(args),
            Some(StdlibFeature::SifrRuntime),
        ),
        "encode_utf8" => (bytes::lower_encode_utf8(args), None),
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
        "bytes_to_hex" => (bytes::lower_bytes_to_hex(args), None),
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
        "re_match" => (re::lower_re_match(args), Some(StdlibFeature::Regex)),
        "re_find" => (re::lower_re_find(args), Some(StdlibFeature::Regex)),
        "re_replace" => (re::lower_re_replace(args), Some(StdlibFeature::Regex)),
        "re_findall" => (re::lower_re_findall(args), Some(StdlibFeature::Regex)),
        "re_split" => (re::lower_re_split(args), Some(StdlibFeature::Regex)),
        "re_find_start" => (re::lower_re_find_start(args), Some(StdlibFeature::Regex)),
        "re_find_end" => (re::lower_re_find_end(args), Some(StdlibFeature::Regex)),
        "re_match_flags" => (re::lower_re_match_flags(args), Some(StdlibFeature::Regex)),
        "re_find_flags" => (re::lower_re_find_flags(args), Some(StdlibFeature::Regex)),
        "re_replace_flags" => (re::lower_re_replace_flags(args), Some(StdlibFeature::Regex)),
        "re_findall_flags" => (re::lower_re_findall_flags(args), Some(StdlibFeature::Regex)),
        "re_split_flags" => (re::lower_re_split_flags(args), Some(StdlibFeature::Regex)),
        "sha256" => (hash::lower_sha256(args), Some(StdlibFeature::Sha2)),
        "md5" => (hash::lower_md5(args), Some(StdlibFeature::Md5)),
        "sha256_bytes" => (hashlib::lower_sha256_bytes(args), Some(StdlibFeature::Sha2)),
        "md5_bytes" => (hashlib::lower_md5_bytes(args), Some(StdlibFeature::Md5)),
        "platform_system" => (platform::lower_platform_system(args), None),
        "platform_arch" => (platform::lower_platform_arch(args), None),
        "platform_node" => (platform::lower_platform_node(args), None),
        "platform_release" => (platform::lower_platform_release(args), None),
        "platform_version" => (platform::lower_platform_version(args), None),
        "platform_processor" => (platform::lower_platform_processor(args), None),
        "uuid4" => (uuid::lower_uuid4(args), Some(StdlibFeature::Rand)),
        "uuid3_text" => (uuid::lower_uuid3(args), Some(StdlibFeature::Uuid)),
        "uuid5_text" => (uuid::lower_uuid5(args), Some(StdlibFeature::Uuid)),
        "toml_parse" => (toml::lower_toml_parse(args), Some(StdlibFeature::Toml)),
        "datetime_now" => (
            datetime::lower_datetime_now(args),
            Some(StdlibFeature::Chrono),
        ),
        "datetime_now_struct" => (
            datetime::lower_datetime_now_struct(args),
            Some(StdlibFeature::Chrono),
        ),
        "datetime_format" => (datetime::lower_datetime_format(args), None),
        "datetime_from_timestamp" => (
            datetime::lower_datetime_from_timestamp(args),
            Some(StdlibFeature::Chrono),
        ),
        "sys_exit" => (sys::lower_sys_exit(args), None),
        "sys_version" => (sys::lower_sys_version(args), None),
        "sys_platform" => (sys::lower_sys_platform(args), None),
        "sys_maxsize" => (sys::lower_sys_maxsize(args), None),
        "process_run" => (process::lower_process_run(args), None),
        "process_spawn" => (process::lower_process_spawn(args), None),
        "process_kill" => (process::lower_process_kill(args), None),
        "process_wait" => (process::lower_process_wait(args), None),
        "process_child_stdin" => (process_pipes::lower_process_child_stdin(args), None),
        "process_child_stdout" => (process_pipes::lower_process_child_stdout(args), None),
        "process_child_stderr" => (process_pipes::lower_process_child_stderr(args), None),
        "process_pipe_read_all" => (process_pipes::lower_process_pipe_read_all(args), None),
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
        "process_shell_run" => (process::lower_process_shell_run(args), None),
        "process_shell_output" => (process::lower_process_shell_output(args), None),
        "process_shell_output_text" => (process::lower_process_shell_output_text(args), None),
        "process_shell_output_timeout" => (process::lower_process_shell_output_timeout(args), None),
        "html_escape" => (html::lower_html_escape(args), None),
        "html_unescape" => (html::lower_html_unescape(args), None),
        "calendar_isleap" => (calendar::lower_calendar_isleap(args), None),
        "calendar_weekday" => (calendar::lower_calendar_weekday(args), None),
        "calendar_monthrange" => (calendar::lower_calendar_monthrange(args), None),
        "gzip_compress" => (gzip::lower_gzip_compress(args), Some(StdlibFeature::Flate2)),
        "gzip_decompress" => (
            gzip::lower_gzip_decompress(args),
            Some(StdlibFeature::Flate2),
        ),
        "zip_create" => (zipfile::lower_zip_create(args), Some(StdlibFeature::Zip)),
        "zip_add_file" => (zipfile::lower_zip_add_file(args), Some(StdlibFeature::Zip)),
        "zip_add_file_bytes" => (
            zipfile::lower_zip_add_file_bytes(args),
            Some(StdlibFeature::Zip),
        ),
        "zip_read_file" => (zipfile::lower_zip_read_file(args), Some(StdlibFeature::Zip)),
        "zip_read_file_bytes" => (
            zipfile::lower_zip_read_file_bytes(args),
            Some(StdlibFeature::Zip),
        ),
        "zip_namelist" => (zipfile::lower_zip_namelist(args), Some(StdlibFeature::Zip)),
        "base64_encode" => (
            base64::lower_base64_encode(args),
            Some(StdlibFeature::Base64),
        ),
        "base64_decode" => (
            base64::lower_base64_decode(args),
            Some(StdlibFeature::Base64),
        ),
        "base64_encode_bytes" => (
            base64::lower_base64_encode_bytes(args),
            Some(StdlibFeature::Base64),
        ),
        "base64_decode_bytes" => (
            base64::lower_base64_decode_bytes(args),
            Some(StdlibFeature::Base64),
        ),
        "base64_encode_opts" => (
            base64::lower_base64_encode_opts(args),
            Some(StdlibFeature::Base64),
        ),
        "base64_decode_opts" => (
            base64::lower_base64_decode_opts(args),
            Some(StdlibFeature::Base64),
        ),
        "urlsafe_b64encode" => (
            base64::lower_urlsafe_b64encode(args),
            Some(StdlibFeature::Base64),
        ),
        "urlsafe_b64decode" => (
            base64::lower_urlsafe_b64decode(args),
            Some(StdlibFeature::Base64),
        ),
        "urlsafe_b64encode_bytes" => (
            base64::lower_urlsafe_b64encode_bytes(args),
            Some(StdlibFeature::Base64),
        ),
        "urlsafe_b64decode_bytes" => (
            base64::lower_urlsafe_b64decode_bytes(args),
            Some(StdlibFeature::Base64),
        ),
        "b32encode" => (base32::lower_b32encode(args), None),
        "b32decode" => (base32::lower_b32decode(args), None),
        "b32hexencode" => (base32::lower_b32hexencode(args), None),
        "b32hexdecode" => (base32::lower_b32hexdecode(args), None),
        "sha1" => (hashlib::lower_sha1(args), Some(StdlibFeature::Sha1)),
        "sha1_bytes" => (hashlib::lower_sha1_bytes(args), Some(StdlibFeature::Sha1)),
        "sha512" => (hashlib::lower_sha512(args), Some(StdlibFeature::Sha2)),
        "sha512_bytes" => (hashlib::lower_sha512_bytes(args), Some(StdlibFeature::Sha2)),
        "sha224" => (hashlib::lower_sha224(args), Some(StdlibFeature::Sha2)),
        "sha224_bytes" => (hashlib::lower_sha224_bytes(args), Some(StdlibFeature::Sha2)),
        "sha384" => (hashlib::lower_sha384(args), Some(StdlibFeature::Sha2)),
        "sha384_bytes" => (hashlib::lower_sha384_bytes(args), Some(StdlibFeature::Sha2)),
        "blake2b" => (hashlib::lower_blake2b(args), Some(StdlibFeature::Blake2)),
        "blake2b_bytes" => (
            hashlib::lower_blake2b_bytes(args),
            Some(StdlibFeature::Blake2),
        ),
        "blake2s" => (hashlib::lower_blake2s(args), Some(StdlibFeature::Blake2)),
        "blake2s_bytes" => (
            hashlib::lower_blake2s_bytes(args),
            Some(StdlibFeature::Blake2),
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
