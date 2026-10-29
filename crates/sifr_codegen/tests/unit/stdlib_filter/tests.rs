use super::*;
use std::collections::HashSet;

#[test]
fn canonical_filter_restores_nominal_refs_before_dependency_closure() {
    let timezone = sifr_type_system::stdlib_class_rust_name("sifr.datetime", "timezone");
    let datetime = sifr_type_system::stdlib_class_rust_name("sifr.datetime", "datetime");
    let code = format!(
        r#"
struct timezone {{}}
struct datetime {{}}
impl datetime {{
    fn with_timezone(tz: &timezone) -> datetime {{ datetime {{}} }}
}}
fn root(value: &{datetime}, tz: &{timezone}) -> {datetime} {{
    datetime::with_timezone(tz)
}}
"#
    );
    let filtered = filter_canonical_stdlib_ir_to_needed(
        &code,
        &HashSet::from(["root".to_string()]),
        "sifr.datetime",
        &HashSet::from(["timezone".to_string(), "datetime".to_string()]),
    );

    assert!(filtered.contains("struct timezone"));
    assert!(filtered.contains("struct datetime"));
    assert!(filtered.contains("impl datetime"));
    assert!(!filtered.contains("__SifrStdlib_"));
}

#[test]
fn canonical_stdlib_names_are_sealed_across_declarations_and_uses() {
    let input = r#"
struct FileHandle {}
struct BinaryFileHandle {}
struct TextFileHandle { inner: BinaryFileHandle }
impl FileHandle { fn close(&self) {} }
fn open() -> FileHandle { FileHandle {} }
"#;

    let sealed = seal_canonical_stdlib_names(
        input,
        "sifr.io",
        &HashSet::from([
            "FileHandle".to_string(),
            "BinaryFileHandle".to_string(),
            "TextFileHandle".to_string(),
        ]),
    );

    assert!(sealed.contains("struct __SifrIoFileHandle"));
    assert!(sealed.contains("struct __SifrIoBinaryFileHandle"));
    assert!(sealed.contains("struct __SifrIoTextFileHandle"));
    assert!(sealed.contains("impl __SifrIoFileHandle"));
    assert!(sealed.contains("fn open() -> __SifrIoFileHandle"));
    assert!(!sealed.contains("struct FileHandle"));

    let native = seal_canonical_stdlib_names(
        "struct NativeFileHandle {}",
        "_sifr.fs",
        &HashSet::from(["NativeFileHandle".to_string()]),
    );
    assert!(native.contains("struct __SifrIoNativeFileHandle"));
}

#[test]
fn canonical_stdlib_sealing_uses_exact_module_identity() {
    let input = r#"
struct JsonValue {}
struct JSONDecodeError {}
impl JsonValue { fn parse() -> Result<JsonValue, JSONDecodeError> { todo!() } }
"#;
    let sealed = seal_canonical_stdlib_names(
        input,
        "sifr.json",
        &HashSet::from(["JsonValue".to_string(), "JSONDecodeError".to_string()]),
    );
    let canonical = sifr_type_system::stdlib_class_rust_name("sifr.json", "JsonValue");
    let canonical_error = sifr_type_system::stdlib_class_rust_name("sifr.json", "JSONDecodeError");

    assert!(sealed.contains(&format!("struct {canonical}")));
    assert!(sealed.contains(&format!("impl {canonical}")));
    assert!(sealed.contains(&format!("struct {canonical_error}")));
    assert!(sealed.contains(&canonical_error));

    let csv = seal_canonical_stdlib_names(
        "struct Error {}",
        "sifr.csv",
        &HashSet::from(["Error".to_string()]),
    );
    let config = seal_canonical_stdlib_names(
        "struct Error {}",
        "sifr.configparser",
        &HashSet::from(["Error".to_string()]),
    );
    let csv_error = sifr_type_system::stdlib_class_rust_name("sifr.csv", "Error");
    let config_error = sifr_type_system::stdlib_class_rust_name("sifr.configparser", "Error");
    assert_ne!(csv_error, config_error);
    assert!(csv.contains(&format!("struct {csv_error}")));
    assert!(config.contains(&format!("struct {config_error}")));

    let global = seal_canonical_stdlib_names(
        "struct WorkerError {}",
        "sifr.parallel",
        &HashSet::from(["WorkerError".to_string()]),
    );
    assert!(global.contains("struct WorkerError"));
}

#[test]
fn canonical_stdlib_sealing_preserves_external_qualified_path_segments() {
    let input = r#"
struct Notify {}
fn local() -> Notify { Notify {} }
fn external() -> tokio::sync::Notify { tokio::sync::Notify::new() }
"#;
    let sealed =
        seal_canonical_stdlib_names(input, "sifr.sync", &HashSet::from(["Notify".to_string()]));
    let canonical = sifr_type_system::stdlib_class_rust_name("sifr.sync", "Notify");

    assert!(sealed.contains(&format!("struct {canonical}")));
    assert!(sealed.contains(&format!("fn local() -> {canonical}")));
    assert!(sealed.contains("tokio::sync::Notify"));
    assert!(!sealed.contains(&format!("tokio::sync::{canonical}")));
}
