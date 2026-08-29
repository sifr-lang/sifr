use super::*;
use std::collections::HashSet;

#[test]
fn relocation_keeps_local_child_conversion_into_canonical_parent() {
    let code = r#"
struct Parent {}
struct RelocatedChild {}
struct LocalChild {}

impl Parent {
    fn inherited() {}
}

impl From<String> for Parent {
    fn from(_value: String) -> Self { Parent {} }
}

impl From<RelocatedChild> for Parent {
    fn from(_value: RelocatedChild) -> Self { Parent {} }
}

impl From<LocalChild> for Parent {
    fn from(_value: LocalChild) -> Self { Parent {} }
}
"#;
    let relocated = HashSet::from(["Parent", "RelocatedChild"]);

    let stripped = strip_relocated_rust_items_by_name(
        code,
        &relocated,
        &HashSet::from(["LocalChild".to_string()]),
    )
    .expect("valid compiler-owned Rust");

    assert!(!stripped.contains("struct Parent"));
    assert!(!stripped.contains("struct RelocatedChild"));
    assert!(!stripped.contains("fn inherited"));
    assert!(!stripped.contains("From<String>"));
    assert!(!stripped.contains("From<RelocatedChild>"));
    assert!(stripped.contains("struct LocalChild"));
    assert!(stripped.contains("From<LocalChild> for Parent"));
}

#[test]
fn relocation_rejects_modified_impls_as_canonical_conversions() {
    let code = r#"
struct Parent {}
struct LocalChild {}

impl !From<LocalChild> for Parent {}
"#;
    let relocated = HashSet::from(["Parent"]);

    let stripped = strip_relocated_rust_items_by_name(
        code,
        &relocated,
        &HashSet::from(["LocalChild".to_string()]),
    )
    .expect("valid compiler-owned Rust");

    assert!(stripped.contains("struct LocalChild"));
    assert!(!stripped.contains("impl !From<LocalChild> for Parent"));
}

#[test]
fn filter_keeps_transitive_dependencies_in_item_order() {
    let code = r#"
use std::collections::HashMap;

fn root() {
    helper();
}

fn helper() {
    leaf();
}

fn leaf() {}

fn unused() {}
"#;
    let imported = HashSet::from(["root".to_string()]);
    let filtered = filter_stdlib_ir_to_needed(code, &imported).expect("valid compiler-owned Rust");

    assert!(filtered.contains("use std::collections::HashMap;"));
    assert!(filtered.contains("fn root()"));
    assert!(filtered.contains("fn helper()"));
    assert!(filtered.contains("fn leaf()"));
    assert!(!filtered.contains("fn unused()"));
}

#[test]
fn filter_ignores_name_mentions_in_strings_and_comments() {
    let code = r#"
fn root() {
    let _ = "helper()";
    // helper()
    /* helper() */
}

fn helper() {}
"#;
    let imported = HashSet::from(["root".to_string()]);
    let filtered = filter_stdlib_ir_to_needed(code, &imported).expect("valid compiler-owned Rust");

    assert!(filtered.contains("fn root()"));
    assert!(!filtered.contains("fn helper()"));
}

#[test]
fn filter_tracks_type_level_dependencies_via_identifiers() {
    let code = r#"
struct Node {}

fn root() -> Node {
    Node {}
}
"#;
    let imported = HashSet::from(["root".to_string()]);
    let filtered = filter_stdlib_ir_to_needed(code, &imported).expect("valid compiler-owned Rust");

    assert!(filtered.contains("fn root()"));
    assert!(filtered.contains("struct Node {}"));
}

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
    )
    .expect("valid compiler-owned Rust");

    assert!(filtered.contains("struct timezone"));
    assert!(filtered.contains("struct datetime"));
    assert!(filtered.contains("impl datetime"));
    assert!(!filtered.contains("__SifrStdlib_"));
}

#[test]
fn filter_supports_enum_trait_static_and_pub_items() {
    let code = r#"
pub enum Mode {
    Fast,
}

pub trait Worker {
    fn run(&self) -> i64;
}

pub struct Job {}

impl Worker for Job {
    fn run(&self) -> i64 { JOB_COUNT }
}

pub static JOB_COUNT: i64 = 7;

pub fn root() -> Box<dyn Worker> {
    let _m = Mode::Fast;
    Box::new(Job {})
}
"#;
    let imported = HashSet::from(["root".to_string()]);
    let filtered = filter_stdlib_ir_to_needed(code, &imported).expect("valid compiler-owned Rust");

    assert!(filtered.contains("pub fn root()"));
    assert!(filtered.contains("pub enum Mode"));
    assert!(filtered.contains("pub trait Worker"));
    assert!(filtered.contains("pub struct Job"));
    assert!(filtered.contains("impl Worker for Job"));
    assert!(filtered.contains("pub static JOB_COUNT"));
}

#[test]
fn filter_supports_async_const_unsafe_fn_and_static_mut() {
    let code = r#"
pub static mut COUNTER: i64 = 0;

pub const fn seed() -> i64 {
    COUNTER
}

pub unsafe fn tick() -> i64 {
    COUNTER + seed()
}

pub async fn root() -> i64 {
    tick()
}
"#;
    let imported = HashSet::from(["root".to_string()]);
    let filtered = filter_stdlib_ir_to_needed(code, &imported).expect("valid compiler-owned Rust");

    assert!(filtered.contains("pub async fn root()"));
    assert!(filtered.contains("pub unsafe fn tick()"));
    assert!(filtered.contains("pub const fn seed()"));
    assert!(filtered.contains("pub static mut COUNTER"));
}

#[test]
fn filter_tracks_type_alias_dependencies_and_drops_unused_aliases() {
    let code = r#"
pub struct Node {}

pub type UsedAlias = Node;
pub type UnusedAlias = i64;

pub fn root() -> UsedAlias {
    Node {}
}
"#;
    let imported = HashSet::from(["root".to_string()]);
    let filtered = filter_stdlib_ir_to_needed(code, &imported).expect("valid compiler-owned Rust");

    assert!(filtered.contains("pub fn root() -> UsedAlias"));
    assert!(filtered.contains("pub type UsedAlias = Node;"));
    assert!(filtered.contains("pub struct Node {}"));
    assert!(!filtered.contains("pub type UnusedAlias = i64;"));
}

#[test]
fn filter_avoids_false_positive_from_local_variable_name() {
    let code = r#"
pub fn root() -> i64 {
    let helper = 1;
    helper + 1
}

pub fn helper() -> i64 {
    2
}
"#;
    let imported = HashSet::from(["root".to_string()]);
    let filtered = filter_stdlib_ir_to_needed(code, &imported).expect("valid compiler-owned Rust");
    assert!(filtered.contains("pub fn root()"));
    assert!(!filtered.contains("pub fn helper()"));
}

#[test]
fn filter_tracks_dependencies_used_in_macro_arguments() {
    let code = r#"
fn root() {
    helper();
}

fn helper() -> String {
    format!("v={}", leaf())
}

fn leaf() -> String {
    "ok".to_string()
}

fn unused() {}
"#;
    let imported = HashSet::from(["root".to_string()]);
    let filtered = filter_stdlib_ir_to_needed(code, &imported).expect("valid compiler-owned Rust");

    assert!(filtered.contains("fn root()"));
    assert!(filtered.contains("fn helper()"));
    assert!(filtered.contains("fn leaf()"));
    assert!(!filtered.contains("fn unused()"));
}

#[test]
fn filter_keeps_all_items_for_needed_type_name() {
    let code = r#"
pub struct Builder {}

impl Builder {
    pub fn new() -> Builder {
        Builder {}
    }
}

pub fn root() -> Builder {
    Builder::new()
}
"#;
    let imported = HashSet::from(["root".to_string()]);
    let filtered = filter_stdlib_ir_to_needed(code, &imported).expect("valid compiler-owned Rust");

    assert!(filtered.contains("pub fn root() -> Builder"));
    assert!(filtered.contains("pub struct Builder {}"));
    assert!(filtered.contains("impl Builder"));
}

#[test]
fn dedup_uses_impl_signature_keys() {
    let code = r#"
struct Item {}

impl Item {
    fn a(&self) {}
}

impl Item {
    fn b(&self) {}
}

impl std::fmt::Display for Item {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) }
}
"#;
    let mut emitted = HashSet::new();
    let skip_types = HashSet::new();
    let once =
        dedup_rust_items(code, &mut emitted, &skip_types).expect("valid compiler-owned Rust");
    let twice =
        dedup_rust_items(code, &mut emitted, &skip_types).expect("valid compiler-owned Rust");

    assert!(once.contains("struct Item {}"));
    assert_eq!(once.matches("impl Item").count(), 2);
    assert!(once.contains("fn a(&self)"));
    assert!(once.contains("fn b(&self)"));
    assert!(once.contains("impl std::fmt::Display for Item"));
    assert!(twice.trim().is_empty());
}

#[test]
fn dedup_distinguishes_syn_3_impl_modifiers() {
    let code = r#"
struct Item {}
trait Marker {}

impl Marker for Item {}
impl !Marker for Item {}
"#;
    let mut emitted = HashSet::new();

    let deduplicated =
        dedup_rust_items(code, &mut emitted, &HashSet::new()).expect("valid compiler-owned Rust");

    assert!(deduplicated.contains("impl Marker for Item"));
    assert!(deduplicated.contains("impl !Marker for Item"));
}

#[test]
fn collects_and_strips_shared_prelude_bits() {
    let input = r#"
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::Mutex;

enum SifrFileHandle {
    Reader(std::fs::File),
}

static __SIFR_FILE_HANDLES: std::sync::OnceLock<
    Mutex<HashMap<i64, SifrFileHandle>>
> = std::sync::OnceLock::new();
static __SIFR_NEXT_FILE_HANDLE_ID: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(1);
fn __sifr_next_file_handle_id() -> i64 {
    __SIFR_NEXT_FILE_HANDLE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

struct FileHandle {
    _handle: i64,
}

fn keep_me() {
    let _ = __SIFR_FILE_HANDLES.get();
}
"#;
    let prepared = collect_and_strip_shared_prelude(input).expect("valid compiler-owned Rust");
    assert!(prepared.shared_needs.collections.needs_hashmap);
    assert!(prepared.shared_needs.collections.needs_hashset);
    assert!(prepared.shared_needs.collections.needs_vecdeque);
    assert!(prepared.shared_needs.file_handles.needs_file_handles);
    assert!(
        prepared
            .shared_needs
            .file_handles
            .provides_file_handle_struct
    );
    assert!(
        !prepared
            .stripped_code
            .contains("use std::collections::HashMap;")
    );
    assert!(!prepared.stripped_code.contains("enum SifrFileHandle"));
    assert!(
        !prepared
            .stripped_code
            .contains("static __SIFR_FILE_HANDLES")
    );
    assert!(
        !prepared
            .stripped_code
            .contains("static __SIFR_NEXT_FILE_HANDLE_ID")
    );
    assert!(
        !prepared
            .stripped_code
            .contains("fn __sifr_next_file_handle_id")
    );
    assert!(prepared.stripped_code.contains("fn keep_me()"));
}

#[test]
fn shared_prelude_needs_ignore_comment_mentions() {
    let input = r#"
// HashMap HashSet VecDeque __SIFR_FILE_HANDLES
fn keep_me() {}
"#;
    let prepared = collect_and_strip_shared_prelude(input).expect("valid compiler-owned Rust");
    assert!(!prepared.shared_needs.collections.needs_hashmap);
    assert!(!prepared.shared_needs.collections.needs_hashset);
    assert!(!prepared.shared_needs.collections.needs_vecdeque);
    assert!(!prepared.shared_needs.file_handles.needs_file_handles);
    assert!(prepared.stripped_code.contains("fn keep_me()"));
}

#[test]
fn strips_combined_shared_use_groups() {
    let input = r#"
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Mutex;
fn keep_me() {}
"#;
    let prepared = collect_and_strip_shared_prelude(input).expect("valid compiler-owned Rust");
    assert!(
        !prepared
            .stripped_code
            .contains("std::collections::{HashMap, HashSet, VecDeque}")
    );
    assert!(!prepared.stripped_code.contains("use std::sync::Mutex;"));
    assert!(prepared.stripped_code.contains("fn keep_me()"));
}

#[test]
fn strips_numeric_imports_centralized_by_module_codegen() {
    let input = r#"
use num_bigint::BigInt;
use rust_decimal::Decimal;
use bigdecimal::BigDecimal;
fn keep_me(value: Decimal) -> (BigInt, BigDecimal) { todo!() }
"#;
    let prepared = collect_and_strip_shared_prelude(input).expect("valid compiler-owned Rust");
    assert!(!prepared.stripped_code.contains("use num_bigint::BigInt;"));
    assert!(
        !prepared
            .stripped_code
            .contains("use rust_decimal::Decimal;")
    );
    assert!(
        !prepared
            .stripped_code
            .contains("use bigdecimal::BigDecimal;")
    );
    assert!(prepared.stripped_code.contains("fn keep_me"));
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
    )
    .expect("valid compiler-owned Rust");

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
    )
    .expect("valid compiler-owned Rust");
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
    )
    .expect("valid compiler-owned Rust");
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
    )
    .expect("valid compiler-owned Rust");
    let config = seal_canonical_stdlib_names(
        "struct Error {}",
        "sifr.configparser",
        &HashSet::from(["Error".to_string()]),
    )
    .expect("valid compiler-owned Rust");
    let csv_error = sifr_type_system::stdlib_class_rust_name("sifr.csv", "Error");
    let config_error = sifr_type_system::stdlib_class_rust_name("sifr.configparser", "Error");
    assert_ne!(csv_error, config_error);
    assert!(csv.contains(&format!("struct {csv_error}")));
    assert!(config.contains(&format!("struct {config_error}")));

    let global = seal_canonical_stdlib_names(
        "struct WorkerError {}",
        "sifr.parallel",
        &HashSet::from(["WorkerError".to_string()]),
    )
    .expect("valid compiler-owned Rust");
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
        seal_canonical_stdlib_names(input, "sifr.sync", &HashSet::from(["Notify".to_string()]))
            .expect("valid compiler-owned Rust");
    let canonical = sifr_type_system::stdlib_class_rust_name("sifr.sync", "Notify");

    assert!(sealed.contains(&format!("struct {canonical}")));
    assert!(sealed.contains(&format!("fn local() -> {canonical}")));
    assert!(sealed.contains("tokio::sync::Notify"));
    assert!(!sealed.contains(&format!("tokio::sync::{canonical}")));
}

#[test]
fn external_runtime_crate_paths_are_made_absolute() {
    let input = r#"
use sifr_runtime::SifrInt;
type Resource = sifr_runtime::python::PythonResourceIdentity;
impl sifr_runtime::python::PythonResourceIdentity for Resource {}
fn value() -> SifrInt { sifr_runtime::SifrInt::from_i64(1) }
"#;

    let absolute = absolutize_external_crate_paths(input).expect("valid compiler-owned Rust");

    assert!(absolute.contains("use ::sifr_runtime::SifrInt"));
    assert!(absolute.contains("type Resource = ::sifr_runtime::python::PythonResourceIdentity"));
    assert!(absolute.contains("impl ::sifr_runtime::python::PythonResourceIdentity"));
    assert!(absolute.contains("::sifr_runtime::SifrInt::from_i64"));
}

#[test]
fn parsed_item_reference_detection_ignores_basename_substrings() {
    let source = r#"
struct MyTimeoutError {}
struct Holder { value: __SifrIoNativeFileHandle }
fn label() -> &'static str { "TimeoutError" }
"#;

    assert!(
        rust_source_references_item_name(source, "__SifrIoNativeFileHandle")
            .expect("valid compiler-owned Rust")
    );
    assert!(
        !rust_source_references_item_name(source, "TimeoutError")
            .expect("valid compiler-owned Rust")
    );
}

#[test]
fn item_partition_moves_only_exact_named_items() {
    let source = r#"
struct __SifrIoNativeFileHandle { value: i64 }
impl __SifrIoNativeFileHandle { fn value(&self) -> i64 { self.value } }
struct Holder { value: __SifrIoNativeFileHandle }
"#;
    let names = HashSet::from(["__SifrIoNativeFileHandle"]);

    let (selected, remaining) =
        partition_rust_items_by_name(source, &names).expect("valid compiler-owned Rust");

    assert!(selected.contains("struct __SifrIoNativeFileHandle"));
    assert!(selected.contains("impl __SifrIoNativeFileHandle"));
    assert!(!selected.contains("struct Holder"));
    assert!(remaining.contains("struct Holder"));
    assert!(!remaining.contains("struct __SifrIoNativeFileHandle"));
    assert!(rust_source_defines_item_name(
        &selected,
        "__SifrIoNativeFileHandle"
    ));
    assert!(!rust_source_defines_item_name(&selected, "Holder"));
}

#[test]
fn stdlib_filter_parse_failures_are_structured_codegen_errors() {
    let error = filter_stdlib_ir_to_needed("fn broken(", &HashSet::new())
        .expect_err("invalid compiler-owned Rust must fail");

    assert!(error.message.contains("stdlib IR filter input"));
    assert!(
        error
            .message
            .contains("failed to parse compiler-owned Rust")
    );
}

#[test]
fn stdlib_relocation_parse_failures_are_structured_codegen_errors() {
    let error = strip_relocated_rust_items_by_name("fn broken(", &HashSet::new(), &HashSet::new())
        .expect_err("invalid compiler-owned Rust must fail");

    assert!(error.message.contains("stdlib nominal relocation"));
    assert!(
        error
            .message
            .contains("failed to parse compiler-owned Rust")
    );
}
