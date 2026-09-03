//! Sifr stdlib import namespace policy.
//!
//! This crate identifies bare CPython-shaped imports that must use Sifr's
//! public `sifr.*` namespace.

use sifr_stdlib_manifest::STDLIB_SOURCES;

/// Match data for a bare CPython-style stdlib module name that should be
/// imported through Sifr's `sifr.*` namespace instead.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BareStdlibMatch {
    pub bare_module: String,
    pub matched_tail: String,
    pub suggested_module: String,
    pub exact_public_module_exists: bool,
}

/// Returns bare-stdlib match data when `module_name` names the tail of a
/// canonical public `sifr.*` module, or starts with one.
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
            exact_public_module_exists: public_stdlib_module_exists(suggested_module),
        });
    }
    if public_stdlib_tail_exists(module_name) {
        return Some(BareStdlibMatch {
            bare_module: module_name.to_string(),
            matched_tail: module_name.to_string(),
            suggested_module: format!("sifr.{module_name}"),
            exact_public_module_exists: true,
        });
    }
    let root = module_name.split('.').next()?;
    if !public_stdlib_tail_exists(root) {
        return None;
    }
    let exact_suggestion = format!("sifr.{module_name}");
    let exact_public_module_exists = public_stdlib_module_exists(&exact_suggestion);
    let suggested_module = if exact_public_module_exists {
        exact_suggestion
    } else {
        format!("sifr.{root}")
    };
    Some(BareStdlibMatch {
        bare_module: module_name.to_string(),
        matched_tail: root.to_string(),
        suggested_module,
        exact_public_module_exists,
    })
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

fn public_stdlib_tail_exists(tail: &str) -> bool {
    STDLIB_SOURCES
        .iter()
        .any(|source| source.module.strip_prefix("sifr.") == Some(tail))
}

fn public_stdlib_module_exists(module_name: &str) -> bool {
    STDLIB_SOURCES
        .iter()
        .any(|source| source.module == module_name)
}

#[cfg(test)]
mod tests {
    use super::is_bare_stdlib_tail;

    #[test]
    fn bare_stdlib_tail_matches_exact_public_module() {
        let matched =
            is_bare_stdlib_tail("math").unwrap_or_else(|| panic!("math should match sifr.math"));

        assert_eq!(matched.bare_module, "math");
        assert_eq!(matched.matched_tail, "math");
        assert_eq!(matched.suggested_module, "sifr.math");
        assert!(matched.exact_public_module_exists);
    }

    #[test]
    fn bare_stdlib_tail_matches_root_fallback_for_missing_submodule() {
        let matched = is_bare_stdlib_tail("collections.abc")
            .unwrap_or_else(|| panic!("collections root should match"));

        assert_eq!(matched.bare_module, "collections.abc");
        assert_eq!(matched.matched_tail, "collections");
        assert_eq!(matched.suggested_module, "sifr.collections");
        assert!(!matched.exact_public_module_exists);
    }

    #[test]
    fn bare_stdlib_tail_matches_reserved_text_i18n_cpython_roots() {
        let codecs =
            is_bare_stdlib_tail("codecs").unwrap_or_else(|| panic!("codecs should be reserved"));
        let encodings_utf8 = is_bare_stdlib_tail("encodings.utf_8")
            .unwrap_or_else(|| panic!("encodings should be reserved"));
        let unicodedata = is_bare_stdlib_tail("unicodedata")
            .unwrap_or_else(|| panic!("unicodedata should be reserved"));
        let gettext =
            is_bare_stdlib_tail("gettext").unwrap_or_else(|| panic!("gettext should be reserved"));

        assert_eq!(codecs.suggested_module, "sifr.encoding");
        assert_eq!(encodings_utf8.bare_module, "encodings.utf_8");
        assert_eq!(encodings_utf8.matched_tail, "encodings");
        assert_eq!(encodings_utf8.suggested_module, "sifr.encoding");
        assert_eq!(unicodedata.suggested_module, "sifr.unicode");
        assert_eq!(gettext.suggested_module, "sifr.i18n");
        assert!(codecs.exact_public_module_exists);
        assert!(unicodedata.exact_public_module_exists);
        assert!(gettext.exact_public_module_exists);
    }

    #[test]
    fn bare_stdlib_tail_matches_reserved_concurrency_runtime_roots() {
        let asyncio =
            is_bare_stdlib_tail("asyncio").unwrap_or_else(|| panic!("asyncio should be reserved"));
        let queue =
            is_bare_stdlib_tail("queue").unwrap_or_else(|| panic!("queue should be reserved"));
        let subprocess = is_bare_stdlib_tail("subprocess")
            .unwrap_or_else(|| panic!("subprocess should be reserved"));
        let concurrent_futures = is_bare_stdlib_tail("concurrent.futures")
            .unwrap_or_else(|| panic!("concurrent should be reserved"));
        let multiprocessing = is_bare_stdlib_tail("multiprocessing")
            .unwrap_or_else(|| panic!("multiprocessing should be reserved"));
        let signal =
            is_bare_stdlib_tail("signal").unwrap_or_else(|| panic!("signal should be reserved"));
        let contextlib = is_bare_stdlib_tail("contextlib")
            .unwrap_or_else(|| panic!("contextlib should be reserved"));
        let warnings = is_bare_stdlib_tail("warnings")
            .unwrap_or_else(|| panic!("warnings should be reserved"));

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
        let socket =
            is_bare_stdlib_tail("socket").unwrap_or_else(|| panic!("socket should be reserved"));
        let ssl = is_bare_stdlib_tail("ssl").unwrap_or_else(|| panic!("ssl should be reserved"));
        let select =
            is_bare_stdlib_tail("select").unwrap_or_else(|| panic!("select should be reserved"));
        let selectors = is_bare_stdlib_tail("selectors")
            .unwrap_or_else(|| panic!("selectors should be reserved"));
        let urllib_parse = is_bare_stdlib_tail("urllib.parse")
            .unwrap_or_else(|| panic!("urllib should be reserved"));
        let http_client =
            is_bare_stdlib_tail("http.client").unwrap_or_else(|| panic!("http should be reserved"));
        let socketserver = is_bare_stdlib_tail("socketserver")
            .unwrap_or_else(|| panic!("socketserver should be reserved"));

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
    fn bare_stdlib_tail_ignores_non_stdlib_and_reserved_roots() {
        assert!(is_bare_stdlib_tail("user_math").is_none());
        assert!(is_bare_stdlib_tail("sifr.math").is_none());
        assert!(is_bare_stdlib_tail("_sifr.math").is_none());
        assert!(is_bare_stdlib_tail("typing").is_none());
        assert!(is_bare_stdlib_tail("enum").is_none());
    }
}
