/// External crate roots reserved by compiler-generated Rust paths.
pub const COMPILER_RUST_PATH_ROOTS: &[&str] = &[
    "alloc",
    "bigdecimal",
    "core",
    "num_bigint",
    "pyo3",
    "rayon",
    "rust_decimal",
    "serde",
    "serde_json",
    "sifr_runtime",
    "sifr_stdlib",
    "std",
    "tokio",
];

const CANONICAL_FILE_HANDLE_RUST_NAMES: &[(&str, &str)] = &[
    ("_sifr.fs.NativeFileHandle", "__SifrIoNativeFileHandle"),
    ("sifr.io.FileHandle", "__SifrIoFileHandle"),
    ("sifr.io.BinaryFileHandle", "__SifrIoBinaryFileHandle"),
    ("sifr.io.TextFileHandle", "__SifrIoTextFileHandle"),
];

/// Canonical declarations supplied by the generated global prelude rather
/// than their merged stdlib module. Exemptions are exact declaration
/// identities: a same-basename class in another module remains distinct.
pub const GLOBAL_RUST_NOMINAL_IDENTITIES: &[&str] = &[
    // Built-in task errors are emitted by the compiler's global error prelude.
    "sifr.builtin.CancellationError",
    "sifr.builtin.TimeoutError",
    // These are emitted by the compiler's shared CPU-offload prelude and are
    // therefore global infrastructure even when surfaced through sifr.parallel.
    "sifr.parallel.WorkerRuntimeError",
    "sifr.parallel.WorkerError",
];

#[must_use]
pub fn is_global_rust_nominal_identity(identity: &str) -> bool {
    GLOBAL_RUST_NOMINAL_IDENTITIES.contains(&identity)
}

/// Return a collision-free Rust identifier for a source-declared nominal type.
///
/// `__Sifr*` and external crate roots are compiler-owned Rust namespaces.
/// Source declarations remain legal with those spellings, but are injectively
/// escaped into a disjoint namespace so generated paths never claim their
/// identity.
#[must_use]
pub fn source_class_rust_name(name: &str) -> String {
    if !name.starts_with("__Sifr") && !COMPILER_RUST_PATH_ROOTS.contains(&name) {
        return name.to_string();
    }
    compiler_owned_identifier("__SifrSource_", name)
}

/// Return the Rust identifier for a nominal class reference.
///
/// Checked stdlib modules are merged into one generated Rust namespace. Their
/// canonical declaration identity, rather than their source basename, owns the
/// emitted identifier. User-package identities continue to use their local
/// source name because project codegen preserves module/import boundaries.
#[must_use]
pub fn class_rust_name(identity: Option<&str>, name: &str) -> String {
    if let Some(identity) = identity {
        if let Some((_, rust_name)) = CANONICAL_FILE_HANDLE_RUST_NAMES
            .iter()
            .find(|(canonical, _)| *canonical == identity)
        {
            return (*rust_name).to_string();
        }
        if (identity.starts_with("sifr.") || identity.starts_with("_sifr."))
            && !is_global_rust_nominal_identity(identity)
        {
            return compiler_owned_identifier("__SifrStdlib_", identity);
        }
    }
    source_class_rust_name(name)
}

/// Return the canonical Rust identifier for a class declared by a checked
/// stdlib module.
#[must_use]
pub fn stdlib_class_rust_name(module: &str, name: &str) -> String {
    let identity = format!("{module}.{name}");
    class_rust_name(Some(&identity), name)
}

pub(super) fn compiler_owned_identifier(prefix: &str, identity: &str) -> String {
    let mut escaped = String::with_capacity(prefix.len() + identity.len());
    escaped.push_str(prefix);
    for byte in identity.bytes() {
        if byte.is_ascii_alphanumeric() {
            escaped.push(char::from(byte));
        } else if byte == b'_' {
            escaped.push_str("__");
        } else {
            escaped.push_str("_x");
            push_hex_byte(&mut escaped, byte);
        }
    }
    escaped
}

fn push_hex_byte(target: &mut String, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    target.push(char::from(HEX[usize::from(byte >> 4)]));
    target.push(char::from(HEX[usize::from(byte & 0x0f)]));
}

#[cfg(test)]
mod tests {
    use super::{class_rust_name, source_class_rust_name, stdlib_class_rust_name};

    #[test]
    fn escapes_compiler_namespaces_injectively() {
        assert_eq!(source_class_rust_name("Regular"), "Regular");
        assert_eq!(source_class_rust_name("std"), "__SifrSource_std");
        assert_eq!(source_class_rust_name("tokio"), "__SifrSource_tokio");
        assert_ne!(
            source_class_rust_name("__SifrSource_std"),
            source_class_rust_name("std")
        );
    }

    #[test]
    fn canonical_stdlib_names_are_identity_owned() {
        assert_eq!(
            class_rust_name(Some("sifr.io.FileHandle"), "FileHandle"),
            "__SifrIoFileHandle"
        );
        assert_eq!(
            class_rust_name(Some("sifr.builtin.TimeoutError"), "TimeoutError"),
            "TimeoutError"
        );
        assert_eq!(
            class_rust_name(Some("local.FileHandle"), "FileHandle"),
            "FileHandle"
        );
        assert_ne!(
            class_rust_name(Some("sifr.json.JSONDecodeError"), "JSONDecodeError"),
            "JSONDecodeError"
        );
        assert_ne!(
            class_rust_name(Some("_sifr.python.PythonError"), "PythonError"),
            class_rust_name(None, "PythonError")
        );
        assert_ne!(
            class_rust_name(Some("sifr.csv.Error"), "Error"),
            class_rust_name(Some("sifr.configparser.Error"), "Error")
        );
        assert_eq!(
            class_rust_name(Some("sifr.json.JsonValue"), "JsonValue"),
            stdlib_class_rust_name("sifr.json", "JsonValue")
        );
        assert_ne!(
            class_rust_name(Some("sifr.json.JsonValue"), "JsonValue"),
            class_rust_name(None, "JsonValue")
        );
    }
}
