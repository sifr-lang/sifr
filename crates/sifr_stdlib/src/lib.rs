//! Sifr Intrinsic Type Registry
//!
//! Defines type signatures for all `_sifr.*` intrinsic modules.
//! These are compiler-provided primitives that map directly to Rust code.
//! User-facing stdlib modules live in `lib/sifr/*.sifr` files.

use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

mod collections_bytes_time;
mod crypto_regex_uuid;
mod features;
mod i18n_core;
mod io_json;
mod ipc_connection;
mod ipc_frame;
mod ipc_payload;
mod ipc_request_tracker;
mod ipc_schema;
mod ipc_transport;
mod math_test;
mod net;
mod platform_misc;
mod process;
mod runtime;
mod signal;
mod sources;
mod sys_fs;
mod task;
mod text_encoding;
mod unicode_core;

use collections_bytes_time::{intrinsic_bytes, intrinsic_collections, intrinsic_time};
use crypto_regex_uuid::{intrinsic_crypto, intrinsic_regex, intrinsic_uuid};
pub use features::{
    feature_for_codegen_requirement, features_for_stdlib_module, generated_cargo_dependencies,
    GeneratedCargoDependency, StdlibFeature, StdlibFeatureSpec, STDLIB_FEATURE_SPECS,
};
use i18n_core::intrinsic_i18n;
use io_json::{intrinsic_io, intrinsic_json};
pub use ipc_connection::{
    negotiate_protocol_version, schema_ranges_overlap, schemas_match_exact, IpcConnectionConfig,
    IpcConnectionError, IpcConnectionPhase, IpcConnectionState, IpcHandshakeDecision,
};
pub use ipc_frame::{
    decode_frame, encode_frame, IpcEnvelope, IpcFrameError, IpcMalformedKind, IpcRejectReason,
    IpcShutdownMode, IpcTerminationReason, IpcWireFrameKind, IpcWireSchema, IpcWorkerState,
    IPC_DEFAULT_MAX_FRAME_BYTES, IPC_LENGTH_PREFIX_BYTES,
};
pub use ipc_payload::{validate_ipc_payload_type, IpcPayloadEligibilityError};
pub use ipc_request_tracker::{IpcRequestTracker, IpcRequestTrackerError, IpcRequestTrackerState};
pub use ipc_schema::{
    canonical_schema_descriptor, fnv1a_128, schema_hash_hex_v1, schema_hash_v1,
    IpcSchemaDescriptor, IpcSchemaField, IpcSchemaType, IpcSchemaVariant,
};
pub use ipc_transport::{read_frame, write_frame, IpcTransportError};
use math_test::{intrinsic_math, intrinsic_test};
use net::intrinsic_net;
use platform_misc::{
    intrinsic_calendar, intrinsic_compress, intrinsic_datetime, intrinsic_html, intrinsic_logging,
    intrinsic_platform, intrinsic_toml,
};
use process::intrinsic_process;
use runtime::intrinsic_runtime;
use signal::intrinsic_signal;
pub use sources::{StdlibSource, STDLIB_SOURCES};
use sys_fs::{intrinsic_fs, intrinsic_sys};
use task::intrinsic_task;
use text_encoding::intrinsic_encoding;
use unicode_core::intrinsic_unicode;

/// Match data for a bare CPython-style stdlib module name that should be
/// imported through Sifr's `sifr.*` namespace instead.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BareStdlibMatch {
    pub bare_module: String,
    pub matched_tail: String,
    pub suggested_module: String,
    pub exact_embedded_module_exists: bool,
}

/// Match data for a CPython-shaped `sifr.*` module that is no longer a public
/// compatibility adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyStdlibModule {
    pub legacy_module: &'static str,
    pub suggested_module: &'static str,
    pub reason: &'static str,
}

/// An intrinsic module definition with its functions and constants.
pub struct IntrinsicModule {
    pub functions: HashMap<String, FunctionType>,
    pub constants: HashMap<String, Type>,
}

/// Helper: construct a built-in error class type (e.g., `IOError`, `ParseError`).
/// Built-in error classes have a single `message: str` field.
fn error_class(name: &str) -> Type {
    Type::Class {
        name: name.to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: vec![],
        parent_class: None,
    }
}

/// Helper: construct Result[T, E] where E is a built-in error class.
fn result_ty(ok: Type, error_name: &str) -> Type {
    Type::Result(Box::new(ok), Box::new(error_class(error_name)))
}

/// Look up an intrinsic module by its dotted name (e.g., "_sifr.io").
/// Returns None if the module is not a known intrinsic module.
pub fn get_intrinsic_module(module_name: &str) -> Option<IntrinsicModule> {
    match module_name {
        "_sifr.io" => Some(intrinsic_io()),
        "_sifr.json" => Some(intrinsic_json()),
        "_sifr.sys" => Some(intrinsic_sys()),
        "_sifr.fs" => Some(intrinsic_fs()),
        "_sifr.math" => Some(intrinsic_math()),
        "_sifr.test" => Some(intrinsic_test()),
        "_sifr.collections" => Some(intrinsic_collections()),
        "_sifr.bytes" => Some(intrinsic_bytes()),
        "_sifr.encoding" => Some(intrinsic_encoding()),
        "_sifr.unicode" => Some(intrinsic_unicode()),
        "_sifr.i18n" => Some(intrinsic_i18n()),
        "_sifr.time" => Some(intrinsic_time()),
        "_sifr.crypto" => Some(intrinsic_crypto()),
        "_sifr.regex" => Some(intrinsic_regex()),
        "_sifr.uuid" => Some(intrinsic_uuid()),
        "_sifr.platform" => Some(intrinsic_platform()),
        "_sifr.net" => Some(intrinsic_net()),
        "_sifr.process" => Some(intrinsic_process()),
        "_sifr.signal" => Some(intrinsic_signal()),
        "_sifr.runtime" => Some(intrinsic_runtime()),
        "_sifr.task" => Some(intrinsic_task()),
        "_sifr.toml" => Some(intrinsic_toml()),
        "_sifr.datetime" => Some(intrinsic_datetime()),
        "_sifr.html" => Some(intrinsic_html()),
        "_sifr.calendar" => Some(intrinsic_calendar()),
        "_sifr.compress" => Some(intrinsic_compress()),
        "_sifr.logging" => Some(intrinsic_logging()),
        _ => None,
    }
}

/// Check if a module name is an intrinsic module.
pub fn is_intrinsic_module(module_name: &str) -> bool {
    module_name.starts_with("_sifr.")
}

/// Check if a module name is a user-facing stdlib module.
pub fn is_stdlib_module(module_name: &str) -> bool {
    module_name.starts_with("sifr.")
}

/// Returns bare-stdlib match data when `module_name` names the tail of an
/// embedded `sifr.*` module, or starts with one.
pub fn is_bare_stdlib_tail(module_name: &str) -> Option<BareStdlibMatch> {
    if module_name.is_empty() || module_name.starts_with("sifr.") || module_name.starts_with('_') {
        return None;
    }
    if let Some(suggested_module) = cpython_stdlib_reserved_suggestion(module_name) {
        let matched_tail = module_name.split('.').next().unwrap_or(module_name);
        return Some(BareStdlibMatch {
            bare_module: module_name.to_string(),
            matched_tail: matched_tail.to_string(),
            suggested_module: suggested_module.to_string(),
            exact_embedded_module_exists: embedded_stdlib_module_exists(suggested_module),
        });
    }
    if embedded_stdlib_tail_exists(module_name) {
        return Some(BareStdlibMatch {
            bare_module: module_name.to_string(),
            matched_tail: module_name.to_string(),
            suggested_module: format!("sifr.{module_name}"),
            exact_embedded_module_exists: true,
        });
    }
    let root = module_name.split('.').next()?;
    if !embedded_stdlib_tail_exists(root) {
        return None;
    }
    let exact_suggestion = format!("sifr.{module_name}");
    let exact_embedded_module_exists = embedded_stdlib_module_exists(&exact_suggestion);
    let suggested_module = if exact_embedded_module_exists {
        exact_suggestion
    } else {
        format!("sifr.{root}")
    };
    Some(BareStdlibMatch {
        bare_module: module_name.to_string(),
        matched_tail: root.to_string(),
        suggested_module,
        exact_embedded_module_exists,
    })
}

/// Returns legacy-module match data for CPython-shaped Sifr stdlib names that
/// are intentionally not public adapters.
pub fn unsupported_legacy_stdlib_module(module_name: &str) -> Option<LegacyStdlibModule> {
    let canonical = match module_name {
        "sifr.asyncio" => "sifr.asyncio",
        "sifr.concurrent" | "sifr.concurrent.futures" => "sifr.concurrent",
        "sifr.contextlib" => "sifr.contextlib",
        "sifr.http.client" => "sifr.http.client",
        "sifr.http.server" => "sifr.http.server",
        "sifr.multiprocessing" => "sifr.multiprocessing",
        "sifr.queue" => "sifr.queue",
        "sifr.select" => "sifr.select",
        "sifr.selectors" => "sifr.selectors",
        "sifr.socket" => "sifr.socket",
        "sifr.socketserver" => "sifr.socketserver",
        "sifr.ssl" => "sifr.ssl",
        "sifr.subprocess" => "sifr.subprocess",
        "sifr.threading" => "sifr.threading",
        "sifr.urllib" | "sifr.urllib.parse" | "sifr.urllib.request" => "sifr.urllib",
        "sifr.warnings" => "sifr.warnings",
        _ => return None,
    };
    legacy_stdlib_module_info(canonical)
}

fn legacy_stdlib_module_info(module_name: &str) -> Option<LegacyStdlibModule> {
    match module_name {
        "sifr.asyncio" => Some(LegacyStdlibModule {
            legacy_module: "sifr.asyncio",
            suggested_module: "sifr.task",
            reason: "structured tasks are exposed through the native task model",
        }),
        "sifr.concurrent" => Some(LegacyStdlibModule {
            legacy_module: "sifr.concurrent",
            suggested_module: "sifr.runtime",
            reason: "executor-style offload is replaced by scoped runtime and parallel work APIs",
        }),
        "sifr.contextlib" => Some(LegacyStdlibModule {
            legacy_module: "sifr.contextlib",
            suggested_module: "sifr.resource",
            reason: "cleanup uses deterministic Sifr resource scopes, not contextlib adapters",
        }),
        "sifr.http.client" => Some(LegacyStdlibModule {
            legacy_module: "sifr.http.client",
            suggested_module: "sifr.http",
            reason: "HTTP client policy belongs to a future Sifr-native client phase, not a CPython-shaped adapter",
        }),
        "sifr.http.server" => Some(LegacyStdlibModule {
            legacy_module: "sifr.http.server",
            suggested_module: "sifr.http",
            reason: "server framework behavior belongs to Phase 41 over the Sifr HTTP substrate",
        }),
        "sifr.multiprocessing" => Some(LegacyStdlibModule {
            legacy_module: "sifr.multiprocessing",
            suggested_module: "sifr.ipc",
            reason: "process workers require the typed IPC design gate",
        }),
        "sifr.queue" => Some(LegacyStdlibModule {
            legacy_module: "sifr.queue",
            suggested_module: "sifr.sync",
            reason: "queue-like communication uses native bounded channels and synchronization",
        }),
        "sifr.select" => Some(LegacyStdlibModule {
            legacy_module: "sifr.select",
            suggested_module: "sifr.net",
            reason: "manual selector readiness is internal; use async network streams",
        }),
        "sifr.selectors" => Some(LegacyStdlibModule {
            legacy_module: "sifr.selectors",
            suggested_module: "sifr.net",
            reason: "manual selector readiness is internal; use async network streams",
        }),
        "sifr.socket" => Some(LegacyStdlibModule {
            legacy_module: "sifr.socket",
            suggested_module: "sifr.net",
            reason: "descriptor-shaped sockets are replaced by typed async network streams",
        }),
        "sifr.socketserver" => Some(LegacyStdlibModule {
            legacy_module: "sifr.socketserver",
            suggested_module: "sifr.http",
            reason: "handler-subclass server adapters are rejected; server products build on the Sifr HTTP substrate",
        }),
        "sifr.ssl" => Some(LegacyStdlibModule {
            legacy_module: "sifr.ssl",
            suggested_module: "sifr.tls",
            reason: "TLS is exposed through typed Sifr TLS configuration and streams",
        }),
        "sifr.subprocess" => Some(LegacyStdlibModule {
            legacy_module: "sifr.subprocess",
            suggested_module: "sifr.process",
            reason: "process management is owned by the native process API",
        }),
        "sifr.threading" => Some(LegacyStdlibModule {
            legacy_module: "sifr.threading",
            suggested_module: "sifr.runtime",
            reason: "threads are an internal substrate for scoped offload, not a public module",
        }),
        "sifr.urllib" => Some(LegacyStdlibModule {
            legacy_module: "sifr.urllib",
            suggested_module: "sifr.url",
            reason: "URL parsing and building are exposed through typed Sifr URL primitives",
        }),
        "sifr.warnings" => Some(LegacyStdlibModule {
            legacy_module: "sifr.warnings",
            suggested_module: "sifr.runtime",
            reason: "Python global warning filters are replaced by typed diagnostics and runtime observability",
        }),
        _ => None,
    }
}

fn cpython_stdlib_reserved_suggestion(module_name: &str) -> Option<&'static str> {
    let root = module_name.split('.').next().unwrap_or(module_name);
    match root {
        "asyncio" => Some("sifr.task"),
        "queue" => Some("sifr.sync"),
        "subprocess" => Some("sifr.process"),
        "concurrent" => Some("sifr.runtime"),
        "multiprocessing" => Some("sifr.ipc"),
        "threading" => Some("sifr.runtime"),
        "signal" => Some("sifr.signal"),
        "contextlib" => Some("sifr.resource"),
        // Python warnings global filters are rejected; structured diagnostics
        // are emitted through Sifr's native runtime observability surface.
        "warnings" => Some("sifr.runtime"),
        "codecs" | "encodings" => Some("sifr.encoding"),
        "unicodedata" => Some("sifr.unicode"),
        "locale" | "gettext" => Some("sifr.i18n"),
        "socket" => Some("sifr.net"),
        "ssl" => Some("sifr.tls"),
        "select" | "selectors" => Some("sifr.net"),
        "urllib" => Some("sifr.url"),
        "http" | "socketserver" => Some("sifr.http"),
        _ => None,
    }
}

fn embedded_stdlib_tail_exists(tail: &str) -> bool {
    STDLIB_SOURCES
        .iter()
        .any(|source| source.module.strip_prefix("sifr.") == Some(tail))
}

fn embedded_stdlib_module_exists(module_name: &str) -> bool {
    STDLIB_SOURCES
        .iter()
        .any(|source| source.module == module_name)
}

#[cfg(test)]
mod tests {
    use super::{
        get_intrinsic_module, is_bare_stdlib_tail, is_intrinsic_module, is_stdlib_module,
        unsupported_legacy_stdlib_module, STDLIB_SOURCES,
    };

    #[test]
    fn known_intrinsic_module_has_signatures() {
        let module = get_intrinsic_module("_sifr.io").expect("_sifr.io should be registered");

        assert!(module.functions.contains_key("read_text"));
        assert!(module.constants.is_empty());
    }

    #[test]
    fn unknown_intrinsic_module_is_not_registered() {
        assert!(get_intrinsic_module("_sifr.not_real").is_none());
    }

    #[test]
    fn stdlib_source_inventory_contains_user_modules() {
        let json = STDLIB_SOURCES
            .iter()
            .find(|source| source.module == "sifr.json")
            .expect("sifr.json should be embedded in the stdlib inventory");

        assert!(json.source.contains("from _sifr.json import"));
    }

    #[test]
    fn module_classification_keeps_intrinsic_and_user_surfaces_separate() {
        assert!(is_intrinsic_module("_sifr.io"));
        assert!(!is_intrinsic_module("sifr.io"));
        assert!(is_stdlib_module("sifr.io"));
        assert!(!is_stdlib_module("_sifr.io"));
    }

    #[test]
    fn bare_stdlib_tail_matches_exact_embedded_module() {
        let matched = is_bare_stdlib_tail("math").expect("math should match sifr.math");

        assert_eq!(matched.bare_module, "math");
        assert_eq!(matched.matched_tail, "math");
        assert_eq!(matched.suggested_module, "sifr.math");
        assert!(matched.exact_embedded_module_exists);
    }

    #[test]
    fn bare_stdlib_tail_matches_root_fallback_for_missing_submodule() {
        let matched =
            is_bare_stdlib_tail("collections.abc").expect("collections root should match");

        assert_eq!(matched.bare_module, "collections.abc");
        assert_eq!(matched.matched_tail, "collections");
        assert_eq!(matched.suggested_module, "sifr.collections");
        assert!(!matched.exact_embedded_module_exists);
    }

    #[test]
    fn bare_stdlib_tail_matches_reserved_text_i18n_cpython_roots() {
        let codecs = is_bare_stdlib_tail("codecs").expect("codecs should be reserved");
        let encodings_utf8 =
            is_bare_stdlib_tail("encodings.utf_8").expect("encodings should be reserved");
        let unicodedata =
            is_bare_stdlib_tail("unicodedata").expect("unicodedata should be reserved");
        let gettext = is_bare_stdlib_tail("gettext").expect("gettext should be reserved");

        assert_eq!(codecs.suggested_module, "sifr.encoding");
        assert_eq!(encodings_utf8.bare_module, "encodings.utf_8");
        assert_eq!(encodings_utf8.matched_tail, "encodings");
        assert_eq!(encodings_utf8.suggested_module, "sifr.encoding");
        assert_eq!(unicodedata.suggested_module, "sifr.unicode");
        assert_eq!(gettext.suggested_module, "sifr.i18n");
        assert!(codecs.exact_embedded_module_exists);
        assert!(unicodedata.exact_embedded_module_exists);
        assert!(gettext.exact_embedded_module_exists);
    }

    #[test]
    fn bare_stdlib_tail_matches_reserved_concurrency_runtime_roots() {
        let asyncio = is_bare_stdlib_tail("asyncio").expect("asyncio should be reserved");
        let queue = is_bare_stdlib_tail("queue").expect("queue should be reserved");
        let subprocess = is_bare_stdlib_tail("subprocess").expect("subprocess should be reserved");
        let concurrent_futures =
            is_bare_stdlib_tail("concurrent.futures").expect("concurrent should be reserved");
        let multiprocessing =
            is_bare_stdlib_tail("multiprocessing").expect("multiprocessing should be reserved");
        let signal = is_bare_stdlib_tail("signal").expect("signal should be reserved");
        let contextlib = is_bare_stdlib_tail("contextlib").expect("contextlib should be reserved");
        let warnings = is_bare_stdlib_tail("warnings").expect("warnings should be reserved");

        assert_eq!(asyncio.suggested_module, "sifr.task");
        assert_eq!(queue.suggested_module, "sifr.sync");
        assert_eq!(subprocess.suggested_module, "sifr.process");
        assert_eq!(concurrent_futures.bare_module, "concurrent.futures");
        assert_eq!(concurrent_futures.suggested_module, "sifr.runtime");
        assert_eq!(multiprocessing.suggested_module, "sifr.ipc");
        assert_eq!(signal.suggested_module, "sifr.signal");
        assert_eq!(contextlib.suggested_module, "sifr.resource");
        assert_eq!(warnings.suggested_module, "sifr.runtime");
    }

    #[test]
    fn bare_stdlib_tail_matches_reserved_network_http_roots() {
        let socket = is_bare_stdlib_tail("socket").expect("socket should be reserved");
        let ssl = is_bare_stdlib_tail("ssl").expect("ssl should be reserved");
        let select = is_bare_stdlib_tail("select").expect("select should be reserved");
        let selectors = is_bare_stdlib_tail("selectors").expect("selectors should be reserved");
        let urllib_parse = is_bare_stdlib_tail("urllib.parse").expect("urllib should be reserved");
        let http_client = is_bare_stdlib_tail("http.client").expect("http should be reserved");
        let socketserver =
            is_bare_stdlib_tail("socketserver").expect("socketserver should be reserved");

        assert_eq!(socket.suggested_module, "sifr.net");
        assert_eq!(ssl.suggested_module, "sifr.tls");
        assert_eq!(select.suggested_module, "sifr.net");
        assert_eq!(selectors.suggested_module, "sifr.net");
        assert_eq!(urllib_parse.bare_module, "urllib.parse");
        assert_eq!(urllib_parse.matched_tail, "urllib");
        assert_eq!(urllib_parse.suggested_module, "sifr.url");
        assert_eq!(http_client.bare_module, "http.client");
        assert_eq!(http_client.matched_tail, "http");
        assert_eq!(http_client.suggested_module, "sifr.http");
        assert_eq!(socketserver.suggested_module, "sifr.http");
    }

    #[test]
    fn legacy_concurrency_runtime_modules_are_not_embedded_public_sources() {
        let legacy_modules = [
            ("sifr.asyncio", "sifr.task"),
            ("sifr.queue", "sifr.sync"),
            ("sifr.subprocess", "sifr.process"),
            ("sifr.concurrent", "sifr.runtime"),
            ("sifr.concurrent.futures", "sifr.runtime"),
            ("sifr.contextlib", "sifr.resource"),
            ("sifr.multiprocessing", "sifr.ipc"),
            ("sifr.threading", "sifr.runtime"),
            ("sifr.warnings", "sifr.runtime"),
        ];

        for (legacy, suggested) in legacy_modules {
            let matched =
                unsupported_legacy_stdlib_module(legacy).expect("legacy module should be rejected");
            assert_eq!(matched.suggested_module, suggested);
            assert!(
                !STDLIB_SOURCES
                    .iter()
                    .any(|source| source.module == matched.legacy_module),
                "{legacy} must not be embedded as a public stdlib source"
            );
        }
    }

    #[test]
    fn legacy_network_http_modules_are_not_embedded_public_sources() {
        let legacy_modules = [
            ("sifr.socket", "sifr.net"),
            ("sifr.ssl", "sifr.tls"),
            ("sifr.select", "sifr.net"),
            ("sifr.selectors", "sifr.net"),
            ("sifr.urllib", "sifr.url"),
            ("sifr.urllib.parse", "sifr.url"),
            ("sifr.urllib.request", "sifr.url"),
            ("sifr.http.client", "sifr.http"),
            ("sifr.http.server", "sifr.http"),
            ("sifr.socketserver", "sifr.http"),
        ];

        for (legacy, suggested) in legacy_modules {
            let matched =
                unsupported_legacy_stdlib_module(legacy).expect("legacy module should be rejected");
            assert_eq!(matched.suggested_module, suggested);
            assert!(
                !STDLIB_SOURCES
                    .iter()
                    .any(|source| source.module == matched.legacy_module),
                "{legacy} must not be embedded as a public stdlib source"
            );
        }
    }

    #[test]
    fn legacy_subprocess_intrinsics_are_not_registered() {
        let sys = get_intrinsic_module("_sifr.sys").expect("_sifr.sys should exist");

        for removed in [
            "subprocess_run",
            "subprocess_run_with_input",
            "subprocess_run_structured",
        ] {
            assert!(
                !sys.functions.contains_key(removed),
                "{removed} must stay removed; use _sifr.process intrinsics instead"
            );
        }
    }

    #[test]
    fn bare_stdlib_tail_ignores_non_stdlib_and_reserved_roots() {
        assert!(is_bare_stdlib_tail("user_math").is_none());
        assert!(is_bare_stdlib_tail("sifr.math").is_none());
        assert!(is_bare_stdlib_tail("_sifr.math").is_none());
        assert!(is_bare_stdlib_tail("typing").is_none());
        assert!(is_bare_stdlib_tail("enum").is_none());
    }
}
