use std::fmt::Write as _;

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
    let mut escaped = String::from("__SifrSource_");
    for byte in name.as_bytes() {
        let _ = write!(escaped, "{byte:02x}");
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::source_class_rust_name;

    #[test]
    fn escapes_compiler_namespaces_injectively() {
        assert_eq!(source_class_rust_name("Regular"), "Regular");
        assert_eq!(source_class_rust_name("std"), "__SifrSource_737464");
        assert_eq!(source_class_rust_name("tokio"), "__SifrSource_746f6b696f");
        assert_ne!(
            source_class_rust_name("__SifrSource_737464"),
            source_class_rust_name("std")
        );
    }
}
